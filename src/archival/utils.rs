/// Look at the `status_ext` of the status response from IA, and reeturn true if the error is "permanent", and the url shouldn't be retried
pub fn check_if_permanent_error(status_ext: &str) -> bool {
    let permanent_errors = vec![
        "error:bad-request",
        "error:blocked-url",
        "error:blocked",
        "error:blocked-client-ip",
        "error:filesize-limit",
        "error:http-version-not-supported",
        "error:invalid-url-syntax",
        "error:invalid-host-resolution",
        "error:method-not-allowed",
        "error:not-implemented",
        "error:not-found",
        "error:no-access",
        "error:unauthorized",
    ];
    permanent_errors
        .iter()
        .any(|&error| status_ext.contains(error))
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
