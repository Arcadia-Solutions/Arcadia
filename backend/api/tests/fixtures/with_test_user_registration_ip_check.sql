-- Requires the "with_test_users" fixture.
-- The check of the registration ip of the user 100 made on registration.
UPDATE users
SET registration_ip_check_provider = 'proxycheck',
    registration_ip_country_code = 'FR',
    registration_ip_country = 'France',
    registration_ip_region = 'Île-de-France',
    registration_ip_city = 'Paris',
    registration_ip_latitude = 48.8566,
    registration_ip_longitude = 2.3522,
    registration_ip_asn = 'AS3215',
    registration_ip_organization = 'Orange',
    registration_ip_is_vpn = TRUE,
    registration_ip_is_proxy = FALSE,
    registration_ip_is_tor = FALSE,
    registration_ip_is_hosting = NULL
WHERE id = 100;
