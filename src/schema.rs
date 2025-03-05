// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 25]
        username -> Varchar,
        email -> Text,
        name -> Text,
        password -> Text,
        image_profile -> Nullable<Text>,
    }
}
