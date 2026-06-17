CREATE TYPE activity_action AS ENUM (
    'create_board',
    'update_board',
    'create_list',
    'update_list',
    'update_list_position',
    'delete_list',
    'create_card',
    'update_card',
    'update_card_list',
    'update_card_position',
    'delete_card'
);

CREATE TABLE activities (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    board_id uuid NOT NULL,
    action activity_action NOT NULL,
    target_id uuid NOT NULL,
    data jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT pkey_activities PRIMARY KEY (id),
    CONSTRAINT fkey_activities_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fkey_activities_to_boards FOREIGN KEY (board_id) REFERENCES boards (id) ON DELETE CASCADE
);
