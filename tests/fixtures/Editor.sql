CREATE TABLE editor
(
    id                  SERIAL,
    privs               INTEGER DEFAULT 0
);

INSERT into editor(id, privs) VALUES
(1, 2),
(2, 1024),
(3, 4096)