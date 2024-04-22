# MusicBrainz - External URLs - Internet Archive Service
(Sorry for such a long messy name, will update later ig)
- [Proposal Doc Link](https://docs.google.com/document/d/1Bk66_HFWEA6gBbFfQzIriGGgxxbEIwN1CbVDcz7FTys/edit?usp=sharing)

### Current Implementation (WIP)
1. `poller task`
   - Poll `edit_data` and `edit_note` for URLs.
   - Transformations to required format
   - Save output to `internet_archive_urls` table (to be created).