# Maintaining the project

This doc provides instructions, guidelines and references to maintain the project without running into troubles.

## Schema Guidelines

- The project depends on `musicbrainz_db`, therefore, make sure all the `CREATE TABLE musicbrainz.*` instructions, present in `scripts/sql` scripts are in sync with MusicBrainz database schema.