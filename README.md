# MusicBrainz - External URLs - Internet Archive Service
(Sorry for such a long messy name, will update later ig)
- [Proposal Doc Link](https://docs.google.com/document/d/1Bk66_HFWEA6gBbFfQzIriGGgxxbEIwN1CbVDcz7FTys/edit?usp=sharing)

### Current Implementation (WIP)
1. `poller task`
   - Create a `Poller` implementation which:
     - initialises the `internet_archive_urls` table, where we will store the URLs.
     - Fetches the `edit_note` id and `edit_data` edit column ids from `internet_archive_urls` , from where the polling will start.
   - Poll `edit_data` and `edit_note` for URLs
   - Transformations to required format
   - Save output to `internet_archive_urls` table
2. `archival task`
   - Has 2 parts:
     1. `notifer`
         - Creates a `Notifier` implementation which:
           - Fetches the last unarchived URL row from `internet_archive_urls` table, and start notifying from this row id.
           - Initialises a postgres function `notify_archive_urls`, which takes the `start_id` integer value, from where we start notifying the channel in one go.
         - Reads `internet_archive_urls` table, and notifies the task which will save the URLS, through a channel called `archive_urls`.
     2. `listener`
         - Listens to the `archive_urls` channel, and makes the necessary Wayback Machine API request (The API calls are still to be made).

### Architecture
```
.
├── Cargo.toml // dependencies
├── Dockerfile 
├── README.md
├── scripts //Helpful scripts for development
│   ├── init_db.sh //initialises the external_url_archiver schema
│   ├── reinit_db.sh //Drops the external_url_archiver schema and reinitializes it
│   └── sql //Import sql scripts
│       ├── 001_CreateSchema.sql
│       ├── 002_CreateTable.sql
│       ├── 003_CreateFunction.sql
│       └── 004_CreateTrigger.sql
└── src
    ├── archival // Contains code for archive task
    │   ├── listener.rs // Contains functions related to listening the channel, and saving the URLs in Wayback Machine
    │   ├── mod.rs
    │   ├── notifier.rs // Struct and Implementation of Notifier that notifies the channel
    │   └── utils.rs // Various SQL related functions
    ├── main.rs // Entrypoint
    ├── poller // Module for polling and transforming Edit/Edit Note schema
    │   ├── looper.rs // Methods called by polling task 
    │   ├── mod.rs // Poller implementation
    │   └── utils.rs // Various SQL functions related to polling and transformation logic
    └── structs // Contains sqlx compliant rust structs
        ├── internet_archive_urls.rs
        └── mod.rs

```

### Local setup
> - Make sure musicbrainz db and the required database tables are present.
> - Follow https://github.com/metabrainz/musicbrainz-docker to install the required containers and db dumps.
> - Rename the `.env.example` to `.env`.
> - After ensuring musicbrainz_db is running on port 5432, Run the script `init_db.sh` in scripts dir.

There are 2 methods to run the program:
1. Build the project and run.
    - Make sure rust is installed.
   - ```shell
        cargo build &&
        ./target/debug/mb-exurl-ia-service
        ```
2. Use the Dockerfile
   - Note that the container has to run in the same network as musicbrainz db network bridge.
   - ```shell
     docker buildx build -t mb-exurl-ia-service:latest .
     ```
   - ```shell
     docker run --network musicbrainz-docker_default -it mb-exurl-ia-service:latest
     ```