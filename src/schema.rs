table! {
    sing (url) {
        url -> Text,
        added -> Timestamptz,
        added_by -> Text,
        last_access -> Timestamptz,
    }
}

table! {
    url_metadata (url) {
        url -> Text,
        title -> Text,
        author -> Text,
        duration -> Interval,
        start_time -> Interval,
    }
}

table! {
    urls (url) {
        url -> Text,
        last_updated -> Timestamptz,
    }
}

joinable!(url_metadata -> urls (url));

allow_tables_to_appear_in_same_query!(
    sing,
    url_metadata,
    urls,
);
