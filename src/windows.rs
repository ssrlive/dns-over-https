#![cfg(windows)]

windows_service::define_windows_service!(ffi_service_main, my_service_main);

pub fn start_service() -> Result<(), windows_service::Error> {
    // Register generated `ffi_service_main` with the system and start the service,
    // blocking this thread until the service is stopped.
    windows_service::service_dispatcher::start("dns-over-tls", ffi_service_main)?;
    Ok(())
}

fn my_service_main(arguments: Vec<std::ffi::OsString>) {
    // The entry point where execution will start on a background thread after a call to
    // `service_dispatcher::start` from `main`.

    if let Err(_e) = run_service(arguments) {
        // Handle errors in some way.
    }
}

fn run_service(_arguments: Vec<std::ffi::OsString>) -> Result<(), crate::BoxError> {
    use windows_service::service::ServiceControl;
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Handle stop event and return control back to the system.
                unsafe { crate::dns_over_tls_stop() };
                ServiceControlHandlerResult::NoError
            }
            // All services must accept Interrogate even if it's a no-op.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register("dns-over-tls", event_handler)?;

    let mut next_status = windows_service::service::ServiceStatus {
        // Should match the one from system service registry
        service_type: windows_service::service::ServiceType::OWN_PROCESS,
        // The new state
        current_state: windows_service::service::ServiceState::Running,
        // Accept stop events when running
        controls_accepted: windows_service::service::ServiceControlAccept::STOP,
        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: windows_service::service::ServiceExitCode::Win32(0),
        // Only used for pending states, otherwise must be zero
        checkpoint: 0,
        // Only used for pending states, otherwise must be zero
        wait_hint: std::time::Duration::default(),
        // Unused for setting status
        process_id: None,
    };

    // Tell the system that the service is running now
    status_handle.set_service_status(next_status.clone())?;

    let args = crate::Args::parse();

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async {
        crate::main_loop(&args).await?;
        Ok::<(), crate::Error>(())
    })?;

    // Tell the system that the service is stopped now
    next_status.current_state = windows_service::service::ServiceState::Stopped;
    status_handle.set_service_status(next_status)?;

    Ok(())
}
