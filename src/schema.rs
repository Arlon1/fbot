table! {
    freiepunkte (id) {
        id -> Int4,
        name -> Text,
    }
}

table! {
    freiepunkte_values (id) {
        id -> Int4,
        userid -> Nullable<Int4>,
        wert -> Int4,
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
    ping (id) {
        id -> Int4,
        sender -> Nullable<Int4>,
        receiver -> Text,
        sent -> Timestamptz,
        scheduled -> Nullable<Timestamptz>,
        message -> Text,
    }
}

table! {
    qedmitglied (userid) {
        userid -> Int4,
        username -> Text,
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
        start_time -> Nullable<Int8>,
    }
}

joinable!(freiepunkte_values -> freiepunkte (id));
joinable!(freiepunkte_values -> qedmitglied (userid));
joinable!(nickname__ -> qedmitglied (userid));
joinable!(nickname_preferred -> nickname__ (preferred));
joinable!(nickname_preferred -> qedmitglied (userid));
joinable!(ping -> qedmitglied (sender));
joinable!(url_metadata -> url__ (url));

allow_tables_to_appear_in_same_query!(
  freiepunkte,
  freiepunkte_values,
  nickname__,
  nickname_preferred,
  ping,
  qedmitglied,
  sing,
  url__,
  url_metadata,
);
