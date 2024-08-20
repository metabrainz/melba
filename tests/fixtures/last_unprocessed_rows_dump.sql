--
-- PostgreSQL database dump
--

-- Dumped from database version 12.18 (Debian 12.18-1.pgdg120+2)
-- Dumped by pg_dump version 16.3

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: backup_last_unprocessed_rows; Type: TABLE DATA; Schema: external_url_archiver; Owner: -
--

INSERT INTO external_url_archiver.last_unprocessed_rows (id_column, table_name) VALUES (111451378, 'edit_data');
INSERT INTO external_url_archiver.last_unprocessed_rows (id_column, table_name) VALUES (71025441, 'edit_note');


--
-- PostgreSQL database dump complete
--

