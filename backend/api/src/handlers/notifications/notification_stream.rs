use crate::{middlewares::auth_middleware, Arcadia};
use actix_web::web::{Data, Query};
use actix_web_lab::sse;
use arcadia_common::error::Error;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};
use tokio::sync::broadcast;

// the frontend keeps a single stream per browser (leader tab), so a few per user is plenty
const MAXIMUM_STREAMS_PER_USER: usize = 5;
// every stream holds a socket, so they may use half of the open files the process is allowed
// (`ulimits` in compose.yml), the other half stays free for requests, database connections and files
static MAXIMUM_STREAMS: LazyLock<usize> = LazyLock::new(|| {
    let mut open_files_limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: getrlimit only writes into the struct it is given
    let open_files_soft_limit =
        if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut open_files_limit) } == 0 {
            open_files_limit.rlim_cur
        } else {
            1024
        };
    usize::try_from(open_files_soft_limit / 2).unwrap_or(usize::MAX)
});

/// Holds a stream slot for the lifetime of the stream and releases it when dropped.
struct NotificationStreamSlot {
    notification_streams: Arc<Mutex<HashMap<i32, usize>>>,
    user_id: i32,
}

impl NotificationStreamSlot {
    fn acquire(
        notification_streams: &Arc<Mutex<HashMap<i32, usize>>>,
        user_id: i32,
    ) -> Result<Self, Error> {
        let mut streams = notification_streams.lock().unwrap();
        let total_streams: usize = streams.values().sum();
        let user_streams = streams.get(&user_id).copied().unwrap_or(0);
        if user_streams >= MAXIMUM_STREAMS_PER_USER || total_streams >= *MAXIMUM_STREAMS {
            return Err(Error::TooManyNotificationStreams);
        }
        streams.insert(user_id, user_streams + 1);
        Ok(Self {
            notification_streams: Arc::clone(notification_streams),
            user_id,
        })
    }
}

impl Drop for NotificationStreamSlot {
    fn drop(&mut self) {
        let mut streams = self.notification_streams.lock().unwrap();
        if let Some(user_streams) = streams.get_mut(&self.user_id) {
            *user_streams -= 1;
            if *user_streams == 0 {
                streams.remove(&self.user_id);
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NotificationStreamQuery {
    pub token: String,
}

pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<NotificationStreamQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<
    sse::Sse<impl futures::Stream<Item = Result<sse::Event, std::convert::Infallible>>>,
    actix_web::Error,
> {
    let user_id = auth_middleware::validate_token::<R>(&query.token, &arc).await?;

    let slot = NotificationStreamSlot::acquire(&arc.notification_streams, user_id)?;
    let mut receiver = arc.notification_sender.subscribe();

    let stream = async_stream::stream! {
        let _slot = slot;
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    if event.user_ids().contains(&user_id) {
                        yield Ok(sse::Event::Data(sse::Data::new(event.event_type())));
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    yield Ok(sse::Event::Data(sse::Data::new("refresh")));
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Ok(sse::Sse::from_stream(stream).with_keep_alive(Duration::from_secs(30)))
}
