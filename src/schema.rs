table! {
    chatuser (userid) {
        userid -> Int4,
        username -> Text,
    }
}

table! {
    nickname__ (nickname) {
        userid -> Int4,
        nickname -> Text,
    }
}

table! {
    nickname_preferred (userid) {
        userid -> Int4,
        preferred -> Nullable<Text>,
    }
}

table! {
    sing (url) {
        url -> Text,
        added -> Timestamptz,
        added_by -> Text,
        last_access -> Timestamptz,
    }
}

table! {
    url__ (url) {
        url -> Text,
        last_updated -> Timestamptz,
    }
}

table! {
    url_metadata (url) {
        url -> Text,
        title -> Nullable<Text>,
        author -> Nullable<Text>,
        duration -> Nullable<Int8>,
    }
}

joinable!(nickname__ -> chatuser (userid));
joinable!(nickname_preferred -> chatuser (userid));
joinable!(nickname_preferred -> nickname__ (preferred));
joinable!(url_metadata -> url__ (url));

allow_tables_to_appear_in_same_query!(
    chatuser,
    nickname__,
    nickname_preferred,
    sing,
    url__,
    url_metadata,
);
