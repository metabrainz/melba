# <div style="text-align: center;">Melba</div>

**<div style="text-align: center;">Musicbrainz External Link wayBack Archiver</div>**

-[Proposal Doc Link](https://docs.google.com/document/d/1Bk66_HFWEA6gBbFfQzIriGGgxxbEIwN1CbVDcz7FTys/edit?usp=sharing)


## About

The project is a rust based service which utilizes Internet Archive's [Wayback Machine](https://web.archive.org/) APIs to preserve URLs present in Musicbrainz database, in Internet Archive history.
MusicBrainz database sees a lot of edits made on a daily basis. With each edit, there’s associated an edit note which provides additional information about the edit. Often, these edit notes, as well as some edits, contain external links, which we want to archive in the Internet Archive.

## Installation

Primary prerequisites:
- Rust
- Postgres
- Docker
- [musicbrainz-docker](https://github.com/metabrainz/musicbrainz-docker) local setup for musicbrainz database
- yq (version >= 4.44.3)

Follow the instructions in [INSTALL.md](docs/INSTALL.md)

## App architecture

For understanding how the project is structured, check [here](docs/ARCHITECTURE.md) 

## Maintenance

Refer to [Maintenance guide](docs/MAINTENANCE.md) for guidelines and instructions for maintaining the project.

## Deployment
Checkout [Deployment Guidelines](docs/DEPLOYMENT.md) for deployment related information.

