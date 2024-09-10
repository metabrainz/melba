# Maintaining the project

This doc provides instructions, guidelines and references to maintain the project without running into troubles.

## Local Development

### Database

- The app depends on `musicbrainz_db` as an external database source, which one can instantiate using [musicbrainz-docker](https://github.com/metabrainz/musicbrainz-docker).
- The database runs as a separate container, in a different network.
- Depending on how we want to run the app, we need to configure the database host.
  - If running this archival app as a binary, one needs to expose the port 5432 of the `musicbrainz-db` to the host computer, and then one can use the database's host as `localhost`.
  - If running as a docker container, the docker container name is resolved as the hostname, so the database host becomes the database container name (`musicbrainz-docker-db-1`)
- Make sure to check the database's docker container name by running:
    ```shell
    docker ps | grep musicbrainz-docker_db
   ```

### Configs
- The configs related to the application are set inside `config/` directory.
- To be able to use the app without issues, one must populate the configs correctly.
- The database's host name should also be correctly be provided in `config/development.yaml`.
- Other configs such as sentry DSN URL, Wayback API key and secret, all should be updated in `config/development.yaml`.
- To control the polling, archival, and cleanup tasks' rates, one can override the default values in `config/development.yaml`.

## Schema Guidelines

- Since the project depends on `musicbrainz_db`, therefore, make sure all the `CREATE TABLE musicbrainz.*` instructions, present in `scripts/sql` scripts are in sync with MusicBrainz database schema.

## Monitoring
- While editing the grafana dashboard, make sure to update `grafana/dashboards/metrics-dashboard.json` file with the corresponding changes. 

## Resources

- [Wayback Machine Save Page Now 2 (SPN2) Public API docs](https://docs.google.com/document/d/1Nsv52MvSjbLb2PCpHlat0gkzw0EvtSgpKHu4mk0MnrA/edit#heading=h.1gmodju1d6p0)