DROP INDEX index_boards_on_slug;

CREATE UNIQUE INDEX index_boards_on_user_id_slug ON boards USING btree (user_id, slug);
