ALTER TABLE cards ADD COLUMN cover_image_attachment_id uuid NULL,
ADD CONSTRAINT fkey_cards_to_cover_image_attachments FOREIGN KEY (cover_image_attachment_id) REFERENCES attachments (
    id
) ON DELETE CASCADE;
