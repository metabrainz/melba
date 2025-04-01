use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use sqlx::types::chrono;
use sqlx::PgPool;

use crate::configuration::SETTINGS;

#[derive(sqlx::Type, Debug, Clone, PartialEq, Deserialize)]
#[sqlx(type_name = "external_url_archiver.url_status")]
pub enum ArchivalStatus {
    /// Waiting for the job to be picked up
    Waiting,

    /// Job popped, sending the url to be processed
    Processing,

    /// Waiting for IA to save the url
    WaitingStatus,

    /// Successfully archived
    Archived,

    /// An error occured, and will be retried later
    Errored,

    /// The job failed too many times, and will be ignored.
    Failed,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Clone)]
pub struct InternetArchiveUrl {
    pub id: i32,
    pub url: String,
    pub job_id: Option<String>,
    #[allow(dead_code)]
    pub from_table: Option<String>,
    #[allow(dead_code)]
    pub from_table_id: Option<i32>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,

    pub status: ArchivalStatus,
    pub status_message: Option<String>,

    /// The number of times the url has been submitted to IA for archival
    pub try_count: i32,

    /// The timestamp of when the url can be retried
    pub retry_after: DateTime<Utc>,
}

impl InternetArchiveUrl {
    /// Return true if a row with the provided row id is in the database
    #[allow(dead_code)]
    pub async fn row_exist(conn: &PgPool, row_id: i32) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar(
            "
            SELECT id FROM external_url_archiver.internet_archive_urls
            WHERE id = $1;
        ",
        )
        .bind(row_id)
        .fetch_optional(conn)
        .await
        .map(|opt: Option<i32>| opt.is_some())
    }

    /// Find the first row to take for a new job
    pub async fn find_new_job(
        conn: &PgPool,
        after_id: Option<i32>,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as(
            "
                SELECT DISTINCT ON (id) *
                FROM external_url_archiver.internet_archive_urls
                WHERE 
                    (status = 'Waiting' OR status = 'Errored')
                    AND id > $1
                    AND retry_after >= NOW() 
                    AND created_at + make_interval(secs => $1) < NOW()
                    AND try_count < $2
                ORDER BY created_at
                LIMIT 1
            ",
        )
        .bind(after_id.unwrap_or(0))
        .bind(SETTINGS.archival_task.max_retry)
        .fetch_optional(conn)
        .await
    }

    /// Set a job as processing.
    ///
    /// This also clears the job id to prevent ambiguity whether it's from a previous try or the current one
    pub async fn set_processing(&mut self, conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            UPDATE external_url_archiver.internet_archive_urls 
            SET status = 'Processing', 
                job_id = NULL
            WHERE id = $1
        ",
        )
        .bind(self.id)
        .execute(conn)
        .await?;

        self.status = ArchivalStatus::Processing;
        self.job_id = None;

        Ok(())
    }

    /// Set a job as waiting status.
    pub async fn set_waiting_status(
        &mut self,
        conn: &PgPool,
        job_id: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            UPDATE external_url_archiver.internet_archive_urls 
            SET status = 'WaitingStatus', 
                job_id = $2
            WHERE id = $1
        ",
        )
        .bind(self.id)
        .bind(&job_id)
        .execute(conn)
        .await?;

        self.status = ArchivalStatus::WaitingStatus;
        self.job_id = Some(job_id);

        Ok(())
    }

    /// Set a job as waiting status.
    pub async fn set_archived(&mut self, conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            UPDATE external_url_archiver.internet_archive_urls 
            SET status = 'Archived', 
            WHERE id = $1
        ",
        )
        .bind(self.id)
        .execute(conn)
        .await?;

        self.status = ArchivalStatus::Archived;

        Ok(())
    }

    /// Set a job as errored.
    pub async fn set_errored(&mut self, conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            UPDATE external_url_archiver.internet_archive_urls 
            SET 
                status = 'Errored',
                try_count = $2,
                retry_after = $3
            WHERE id = $1
        ",
        )
        .bind(self.id)
        .bind(self.try_count + 1)
        .bind(self.retry_after + SETTINGS.archival_task.get_retry_interval())
        .execute(conn)
        .await?;

        self.status = ArchivalStatus::Errored;
        self.try_count += 1;
        self.retry_after += SETTINGS.archival_task.get_retry_interval();

        Ok(())
    }

    /// Set a job as failed.
    pub async fn set_failed(&mut self, conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            UPDATE external_url_archiver.internet_archive_urls 
            SET 
                status = 'Failed',
            WHERE id = $1
        ",
        )
        .bind(self.id)
        .bind(self.try_count + 1)
        .bind(self.retry_after + SETTINGS.archival_task.get_retry_interval())
        .execute(conn)
        .await?;

        self.status = ArchivalStatus::Failed;

        Ok(())
    }

    pub async fn get_pending_jobs(conn: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT DISTINCT ON (id) *
            FROM external_url_archiver.internet_archive_urls
            WHERE 
                status = 'WaitingStatus'
        ",
        )
        .fetch_all(conn)
        .await
    }

    /// Delete all the urls that are failed, and have passed the max wait for it
    ///
    /// This waits 30 seconds after the expiration date to make sure the job doesn't get picked up as it expires
    pub async fn delete_errored_and_expired(conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            DELETE FROM external_url_archiver.internet_archive_urls
            WHERE 
                status = 'Errored' AND
                created_at + make_interval(secs => $1 + 30) > NOW()
        ",
        )
        .bind(SETTINGS.archival_task.allow_remove_row_after)
        .execute(conn)
        .await?;

        Ok(())
    }

    /// Delete all the urls that are waiting to be picked up, and have passed the max wait for it
    ///
    /// This waits 30 seconds after the expiration date to make sure the job doesn't get picked up as it expires
    pub async fn delete_waiting_and_expired(conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            DELETE FROM external_url_archiver.internet_archive_urls
            WHERE 
                status = 'Waiting' AND
                created_at + make_interval(secs => $1 + 30) > NOW()
        ",
        )
        .bind(SETTINGS.archival_task.allow_remove_row_after)
        .execute(conn)
        .await?;

        Ok(())
    }

    /// Delete all the URLs that have been failed
    pub async fn delete_failed(conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
            DELETE FROM external_url_archiver.internet_archive_urls
            WHERE 
                status = 'Failed'
        ",
        )
        .bind(SETTINGS.archival_task.allow_remove_row_after)
        .execute(conn)
        .await?;

        Ok(())
    }
}
