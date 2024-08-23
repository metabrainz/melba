DO $$
BEGIN
   -- Insert into musicbrainz.editor if the table exists
   IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'musicbrainz' AND table_name = 'editor') THEN
      INSERT INTO musicbrainz.editor (id, name, privs, email, website, bio, member_since, email_confirm_date, last_login_date, last_updated, birth_date, gender, area, password, ha1, deleted)
      VALUES (1, 'Anonymous', 4096, '', NULL, NULL, NULL, '2009-10-18 18:20:17.333759+00', '2024-05-04 00:17:56.699735+00', '2018-03-15 08:15:36.728395+00', NULL, NULL, NULL, '{CLEARTEXT}mb', 'fad9cdfaf96a2ddb0eceb7b07d269bea', false);
   END IF;

   -- Insert into musicbrainz.edit if the table exists
   IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'musicbrainz' AND table_name = 'edit') THEN
      INSERT INTO musicbrainz.edit (id, editor, type, status, autoedit, open_time, close_time, expire_time, language, quality)
      VALUES (21965, 1, 207, 3, 0, '2000-11-14 11:51:55+00', '2000-11-16 11:51:55+00', '2000-11-16 03:51:55+00', NULL, 1);
   END IF;

   -- Insert into musicbrainz.edit_data if the table exists
   IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'musicbrainz' AND table_name = 'edit_data') THEN
      INSERT INTO musicbrainz.edit_data (edit, data)
      VALUES (21965, '{"new": {"sort_name": "Animals, The"}, "old": {"sort_name": "The Animals"}, "entity": {"id": "1433", "name": "The Animals"}}');
   END IF;

   -- Insert into musicbrainz.edit_note if the table exists
   IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'musicbrainz' AND table_name = 'edit_note') THEN
      INSERT INTO musicbrainz.edit_note (id, editor, edit, text, post_time)
      VALUES (771, 1, 85521, 'This edit moderation clashes with an existing item in the database.', NULL);
   END IF;
END $$;
