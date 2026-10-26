use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq)]
pub enum AnnounceWarning {
    #[error("Stopped peer doesn't exist.")]
    StoppedPeerDoesntExist,
    #[error("Rate limit exceeded. Please wait.")]
    RateLimitExceeded,
    #[error("Download slot limit reached")]
    HitDownloadSlotLimit,
    #[error("Connectivity issue detected. Enable port-forwarding to resolve.")]
    ConnectivityIssueDetected,
}

impl AnnounceWarning {
    /// Returns true if the warning should not be sent to the user
    /// but still used internally to give empty peer lists.
    fn is_silent(&self) -> bool {
        match self {
            // Silenced because it's unlikely qBittorrent will ever fix its
            // duplicate announce bug.
            Self::RateLimitExceeded => true,
            _ => false,
        }
    }
}

const SEPARATOR: &[u8] = b"; ";

pub struct WarningCollection {
    warnings: Vec<AnnounceWarning>,
}

impl WarningCollection {
    /// Initializes a new warning collection.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    /// Calculates the max byte length of the combined warning message.
    #[inline(always)]
    pub const fn max_byte_length(&self) -> usize {
        // Individual warnings should be limited to less than 64 characters
        // (the limit of some clients)
        const MAX_WARNING_LEN: usize = 64 + SEPARATOR.len();

        self.warnings.len() * MAX_WARNING_LEN
    }

    /// Add a new warning to the collection.
    #[inline(always)]
    pub fn add(&mut self, warning: AnnounceWarning) {
        self.warnings.push(warning);
    }

    /// Returns true if there are no warnings.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }

    /// Create the warning message to be returned to the user.
    pub fn into_message(self) -> Option<Vec<u8>> {
        if self.warnings.is_empty() {
            return None;
        }

        let mut message: Vec<u8> = Vec::with_capacity(self.max_byte_length());

        self.warnings.into_iter().for_each(|warning| {
            if !warning.is_silent() {
                message.extend(warning.to_string().as_bytes());
                message.extend(SEPARATOR);
            }
        });

        if message.is_empty() {
            return None;
        }

        // remove the last separator
        message.truncate(message.len() - SEPARATOR.len());

        Some(message)
    }
}

impl Default for WarningCollection {
    fn default() -> Self {
        Self::new()
    }
}
