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
├── src
│   ├── app // Contains implementation of the app which polls and archives
│   │   └── mod.rs
│   ├── archival // Contains code for archive task
│   │   ├── listener.rs  // Contains functions related to listening the channel, and saving the URLs in Wayback Machine
│   │   ├── mod.rs
│   │   ├── notifier.rs  // Struct and Implementation of Notifier that notifies the channel
│   │   ├── tests // Unit tests for archival module
│   │   │   └── utils.rs
│   │   └── utils.rs // Various SQL related functions
│   ├── cli // Module for cli functionality
│   │   ├── args.rs
│   │   ├── mod.rs
│   │   └── utils.rs
│   ├── main.rs // Entrypoint of the app
│   ├── poller // Module for polling and transforming Edit/Edit Note schema
│   │   ├── looper.rs  // Methods called by polling task 
│   │   ├── mod.rs // Poller implementation
│   │   ├── tests // Unit tests for poller task
│   │   │   └── utils.rs
│   │   └── utils.rs  // Various SQL functions related to polling and transformation logic
│   └── structs // Contains sqlx compliant rust structs
│       ├── internet_archive_urls.rs
│       └── mod.rs
└── tests // Contains test db fixtures, and integration tests
    └── fixtures
        ├── EditData.sql
        ├── EditNote.sql
        └── InternetArchiveUrls.sql
```