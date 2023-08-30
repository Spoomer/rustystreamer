// @generated automatically by Diesel CLI.

diesel::table! {
    Collections (collection_id) {
        collection_id -> Integer,
        title -> Text,
        parent_id -> Nullable<Integer>,
    }
}

diesel::table! {
    VideoTimeStamps (video_id) {
        video_id -> Integer,
        timestamp -> Integer,
    }
}

diesel::table! {
    Videos (video_id) {
        video_id -> Integer,
        title -> Text,
        file_name -> Text,
        file_type -> Text,
        collection_id -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    Collections,
    VideoTimeStamps,
    Videos,
);
