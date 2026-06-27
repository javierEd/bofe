CREATE TABLE card_attachments (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    card_id uuid NOT NULL,
    attachment_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT pkey_card_attachments PRIMARY KEY (id),
    CONSTRAINT fkey_card_attachments_to_cards FOREIGN KEY (card_id) REFERENCES cards (
        id
    ) ON DELETE CASCADE,
    CONSTRAINT fkey_card_attachments_to_attachments FOREIGN KEY (attachment_id) REFERENCES attachments (
        id
    ) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_card_attachments_on_card_id_attachment_id ON card_attachments USING btree (
    card_id, attachment_id
);
