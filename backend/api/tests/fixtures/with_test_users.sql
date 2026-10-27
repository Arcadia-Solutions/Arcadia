-- Basic user for read operations and permission denial tests
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (100, 'user_basic', 'test_user@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3837', 'newbie', 'arcadia', '{download_torrent,upload_torrent,create_torrent_request,create_forum_thread,create_forum_post,create_forum_poll_vote}');

-- User with edit_artist permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (101, 'user_edit_art', 'test_user_edit_artist@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3838', 'newbie', 'arcadia', '{edit_artist,create_forum_thread,create_forum_post,upload_torrent,create_torrent_request,create_forum_poll_vote}');

-- User with edit_series permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (102, 'user_edit_ser', 'test_user_edit_series@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3839', 'newbie', 'arcadia', '{edit_series}');

-- User with edit_title_group_comment permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (103, 'user_edit_tgc', 'test_user_edit_title_group_comment@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c383a', 'newbie', 'arcadia', '{edit_title_group_comment}');

-- User with create_css_sheet permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (104, 'user_css_crt', 'test_user_create_css_sheet@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c383b', 'newbie', 'arcadia', '{create_css_sheet}');

-- User with edit_css_sheet permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (105, 'user_css_edit', 'test_user_edit_css_sheet@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c383c', 'newbie', 'arcadia', '{edit_css_sheet}');

-- User with create_forum_category permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (107, 'user_cat_crt', 'test_user_create_forum_category@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c383e', 'newbie', 'arcadia', '{create_forum_category}');

-- User with edit_forum_category permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (108, 'user_cat_edit', 'test_user_edit_forum_category@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c383f', 'newbie', 'arcadia', '{edit_forum_category}');

-- User with create_forum_sub_category permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (109, 'user_sub_crt', 'test_user_create_forum_sub_category@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3840', 'newbie', 'arcadia', '{create_forum_sub_category}');

-- User with edit_forum_sub_category permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (110, 'user_sub_edit', 'test_user_edit_forum_sub_category@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3841', 'newbie', 'arcadia', '{edit_forum_sub_category}');

-- User with edit_forum_thread permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (111, 'user_thr_edit', 'test_user_edit_forum_thread@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3842', 'newbie', 'arcadia', '{edit_forum_thread}');

-- User with edit_forum_post permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (112, 'user_post_edit', 'test_user_edit_forum_post@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3843', 'newbie', 'arcadia', '{edit_forum_post}');

-- User with both create and edit forum category permissions (for flow tests)
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (113, 'user_cat_flow', 'test_user_cat_flow@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3844', 'newbie', 'arcadia', '{create_forum_category,edit_forum_category}');

-- User with both create and edit forum sub category permissions (for flow tests)
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (114, 'user_sub_flow', 'test_user_sub_flow@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3845', 'newbie', 'arcadia', '{create_forum_sub_category,edit_forum_sub_category}');

-- User with create_user_class permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (115, 'user_cls_crt', 'test_user_create_user_class@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3846', 'newbie', 'arcadia', '{create_user_class}');

-- User with edit_user_class permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (116, 'user_cls_edit', 'test_user_edit_user_class@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3847', 'newbie', 'arcadia', '{edit_user_class}');

-- User with delete_user_class permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (117, 'user_cls_del', 'test_user_delete_user_class@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3848', 'newbie', 'arcadia', '{delete_user_class}');

-- User with set_and_view_user_permissions permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (118, 'user_perm_edit', 'test_user_edit_permissions@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3849', 'newbie', 'arcadia', '{set_and_view_user_permissions}');

-- User with lock_user_class permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (119, 'user_lock_cls', 'test_user_lock_class@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c384a', 'newbie', 'arcadia', '{lock_user_class}');

-- User with change_user_class permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (120, 'user_cls_chg', 'test_user_change_class@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c384b', 'newbie', 'arcadia', '{change_user_class}');

-- User with edit_arcadia_settings permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (121, 'user_arc_set', 'test_user_arcadia_settings@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c384c', 'newbie', 'arcadia', '{edit_arcadia_settings}');

-- User with create_donation permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (122, 'user_don_crt', 'test_user_create_donation@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c384d', 'newbie', 'arcadia', '{create_donation}');

-- User with edit_donation permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (123, 'user_don_edit', 'test_user_edit_donation@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c384e', 'newbie', 'arcadia', '{edit_donation}');

-- User with delete_donation permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (124, 'user_don_del', 'test_user_delete_donation@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c384f', 'newbie', 'arcadia', '{delete_donation}');

-- User with search_donation permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (125, 'user_don_srch', 'test_user_search_donation@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3850', 'newbie', 'arcadia', '{search_donation}');

-- User with warn_user and ban_user permissions
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (126, 'user_warn_ban', 'test_user_warn_ban@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3851', 'newbie', 'arcadia', '{warn_user,ban_user}');

-- User with search_unauthorized_access permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (127, 'user_unauth', 'test_user_unauth_search@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3852', 'newbie', 'arcadia', '{search_unauthorized_access}');

-- User with delete_forum_category permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (128, 'user_cat_del', 'test_user_delete_forum_category@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3853', 'newbie', 'arcadia', '{delete_forum_category}');

-- User with delete_forum_sub_category permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (129, 'user_sub_del', 'test_user_delete_forum_sub_category@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3854', 'newbie', 'arcadia', '{delete_forum_sub_category}');

-- User with delete_forum_thread permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (130, 'user_thr_del', 'test_user_delete_forum_thread@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3855', 'newbie', 'arcadia', '{delete_forum_thread}');

-- User with delete_forum_post permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (131, 'user_post_del', 'test_user_delete_forum_post@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3856', 'newbie', 'arcadia', '{delete_forum_post}');

-- User with delete_artist permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (132, 'user_art_del', 'test_user_delete_artist@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3857', 'newbie', 'arcadia', '{delete_artist}');

-- User with set_torrent_staff_checked permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (133, 'user_tor_stfc', 'test_user_set_torrent_staff_checked@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3858', 'newbie', 'arcadia', '{set_torrent_staff_checked}');

-- User with search_user_edit_change_logs permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (134, 'user_edit_log', 'test_user_search_edit_logs@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3859', 'newbie', 'arcadia', '{search_user_edit_change_logs}');

-- User with edit_edition_group permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (135, 'user_edit_eg', 'test_user_edit_edition_group@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c385a', 'newbie', 'arcadia', '{edit_edition_group}');

-- User with edit_collage permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (136, 'user_col_edit', 'test_user_edit_collage@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c385b', 'newbie', 'arcadia', '{edit_collage}');

-- User with delete_collage permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (137, 'user_col_del', 'test_user_delete_collage@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c385c', 'newbie', 'arcadia', '{delete_collage}');

-- User with staff PM permissions (read_staff_pm, reply_staff_pm, resolve_staff_pm)
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (138, 'user_staff_pm', 'test_user_staff_pm@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c385d', 'newbie', 'arcadia', '{read_staff_pm,reply_staff_pm,resolve_staff_pm}');

-- User with view_torrent_peers_and_activities permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (139, 'user_view_peers', 'test_user_view_peers@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c385e', 'newbie', 'arcadia', '{view_torrent_peers_and_activities}');

-- User with edit_torrent_up_down_factors permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (140, 'user_tor_fact', 'test_user_edit_torrent_factors@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c385f', 'newbie', 'arcadia', '{edit_torrent_up_down_factors}');

-- User with delete_title_group permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (141, 'user_tg_del', 'test_user_delete_title_group@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3860', 'newbie', 'arcadia', '{delete_title_group}');

-- User with delete_collage_entry permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (142, 'user_ce_del', 'test_user_delete_collage_entry@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3861', 'newbie', 'arcadia', '{delete_collage_entry}');

-- User with delete_torrent_report permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (143, 'user_tr_del', 'test_user_delete_torrent_report@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3862', 'newbie', 'arcadia', '{delete_torrent_report}');

-- User with remove_title_group_from_series permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (144, 'user_rm_tg_ser', 'test_user_remove_tg_from_series@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3863', 'newbie', 'arcadia', '{remove_title_group_from_series}');

-- User with manage_title_group_tags permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (145, 'user_tag_manage', 'test_user_manage_tags@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3864', 'newbie', 'arcadia', '{manage_title_group_tags}');

-- User with merge_title_group permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (146, 'user_tg_merge', 'test_user_merge_title_group@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3865', 'newbie', 'arcadia', '{merge_title_group}');

-- User with delete_edition_group permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (147, 'user_eg_del', 'test_user_delete_edition_group@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3866', 'newbie', 'arcadia', '{delete_edition_group}');

-- User with move_torrent_to_other_edition_group permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (148, 'user_tor_move', 'test_user_move_torrent@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3867', 'newbie', 'arcadia', '{move_torrent_to_other_edition_group}');

-- User with view_stats_details permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (149, 'user_view_stats', 'test_user_view_stats_details@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3868', 'newbie', 'arcadia', '{view_stats_details}');

-- User with delete_user_edit_change_log permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (150, 'user_del_edlog', 'test_user_delete_edit_logs@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3869', 'newbie', 'arcadia', '{delete_user_edit_change_log,search_user_edit_change_logs}');

-- User with link_similar_wiki_articles permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (151, 'user_link_wiki', 'test_user_link_similar_wiki_articles@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c386a', 'newbie', 'arcadia', '{link_similar_wiki_articles}');

-- User with search_users permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (152, 'user_usr_srch', 'test_user_search_users@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c386b', 'newbie', 'arcadia', '{search_users}');

-- User with generate_reset_password_token permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (153, 'user_pwd_chg', 'test_user_change_password@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c386c', 'newbie', 'arcadia', '{generate_reset_password_token}');

-- User with create_user_badge and create_user_badge_category permissions
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (154, 'user_badge_crt', 'test_user_create_user_badge@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c386d', 'newbie', 'arcadia', '{create_user_badge,create_user_badge_category}');

-- User with delete_user_badge permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (155, 'user_badge_del', 'test_user_delete_user_badge@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c386e', 'newbie', 'arcadia', '{delete_user_badge}');

-- User with view_invisible_user_badges permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (156, 'user_badge_view', 'test_user_view_invisible_user_badges@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c386f', 'newbie', 'arcadia', '{view_invisible_user_badges}');

-- User with award_user_badge permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (157, 'user_badge_awd', 'test_user_award_user_badge@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3870', 'newbie', 'arcadia', '{award_user_badge}');

-- User with revoke_user_badge permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (158, 'user_badge_rev', 'test_user_revoke_user_badge@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3871', 'newbie', 'arcadia', '{revoke_user_badge}');

-- User with read_all_conversations permission, NOT a member of any test conversation
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (159, 'user_ra_third', 'test_user_read_all_conv@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3872', 'newbie', 'arcadia', '{read_all_conversations}');

-- User with read_all_conversations permission, used as a conversation member (sender of conversation 102)
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (160, 'user_ra_membr', 'test_user_read_all_member@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3873', 'newbie', 'arcadia', '{read_all_conversations}');

-- User with manage_site_highlights permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (161, 'user_site_high', 'test_user_manage_site_highlights@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3874', 'newbie', 'arcadia', '{manage_site_highlights}');

-- User with manage_related_forum_thread permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (162, 'user_rel_thr', 'test_user_manage_related_forum_thread@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3875', 'newbie', 'arcadia', '{manage_related_forum_thread}');

-- User with edit_torrent permission but NOT edit_torrent_trumpable
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (163, 'user_edit_tor', 'test_user_edit_torrent@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3876', 'newbie', 'arcadia', '{edit_torrent}');

-- User with both edit_torrent and edit_torrent_trumpable permissions
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (164, 'user_edit_trump', 'test_user_edit_torrent_trumpable@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3877', 'newbie', 'arcadia', '{edit_torrent,edit_torrent_trumpable}');

-- User with unlink_similar_title_group permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (165, 'user_rm_sim_tg', 'test_user_unlink_similar_title_group@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3878', 'newbie', 'arcadia', '{unlink_similar_title_group}');

-- User with link_similar_title_group permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (166, 'user_lnk_sim_tg', 'test_user_link_similar_title_group@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c3879', 'newbie', 'arcadia', '{link_similar_title_group}');

-- User with send_mass_pm permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (167, 'user_mass_pm', 'test_user_send_mass_pm@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c387a', 'newbie', 'arcadia', '{send_mass_pm}');

-- User with delete_title_group_comment permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (168, 'user_del_tgc', 'test_user_delete_title_group_comment@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c387b', 'newbie', 'arcadia', '{delete_title_group_comment}');

-- User with delete_torrent_request_comment permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (169, 'user_del_trc', 'test_user_delete_torrent_request_comment@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c387c', 'newbie', 'arcadia', '{delete_torrent_request_comment}');

-- User with delete_torrent_request permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (170, 'user_del_tr', 'test_user_delete_torrent_request@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c387d', 'newbie', 'arcadia', '{delete_torrent_request}');

-- User with see_paranoia_hidden_user_info permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (171, 'user_paranoia', 'test_user_see_paranoia_hidden_user_info@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c387e', 'newbie', 'arcadia', '{see_paranoia_hidden_user_info}');

-- User with see_foreign_bonus_points_logs permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (172, 'user_foreign_bp', 'test_user_see_foreign_bonus_points_logs@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c387f', 'newbie', 'arcadia', '{see_foreign_bonus_points_logs}');

-- User with react_to_content permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (190, 'user_reactor', 'test_user_react@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f0', 'newbie', 'arcadia', '{react_to_content,create_forum_thread,create_forum_post}');

-- User with view_foreign_invitations permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (191, 'user_frgn_inv', 'test_user_view_foreign_invitations@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f1', 'newbie', 'arcadia', '{view_foreign_invitations}');

-- User with write_user_staff_note permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (192, 'user_wr_note', 'test_user_write_user_staff_note@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f2', 'newbie', 'arcadia', '{write_user_staff_note}');

-- User with edit_user_staff_notes permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (193, 'user_ed_notes', 'test_user_edit_user_staff_notes@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f3', 'newbie', 'arcadia', '{edit_user_staff_notes}');

-- User with view_foreign_user_staff_notes permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (194, 'user_frgn_notes', 'test_user_view_foreign_user_staff_notes@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f4', 'newbie', 'arcadia', '{view_foreign_user_staff_notes,write_user_staff_note}');

-- User with both search_users and set_and_view_user_permissions permissions
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (195, 'user_usr_perm', 'test_user_search_users_with_permissions@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f5', 'newbie', 'arcadia', '{search_users,set_and_view_user_permissions}');

-- User with both send_mass_pm and set_and_view_user_permissions permissions
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (196, 'user_mass_perm', 'test_user_send_mass_pm_with_permissions@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f6', 'newbie', 'arcadia', '{send_mass_pm,set_and_view_user_permissions}');

-- User with request_reseed permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (197, 'user_reseed', 'test_user_request_reseed@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f7', 'newbie', 'arcadia', '{request_reseed}');

-- User with view_registration_ips permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (198, 'user_reg_ips', 'test_user_view_registration_ips@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f8', 'newbie', 'arcadia', '{view_registration_ips}');

-- User with manage_advent_calendars permission
INSERT INTO users (id, username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions)
VALUES (199, 'user_advent_cal', 'test_user_manage_advent_calendars@testdomain.com', '$argon2id$v=19$m=19456,t=2,p=1$WM6V9pJ2ya7+N+NNIUtolg$n128u9idizCHLwZ9xhKaxOttLaAVZZgvfRZlRAnfyKk', '10.10.4.88', 'd2037c66dd3e13044e0d2f9b891c38f9', 'newbie', 'arcadia', '{manage_advent_calendars}');
