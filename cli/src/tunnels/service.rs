/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use std::path::PathBuf;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::commands::tunnels::ShutdownSignal;
use crate::log;
use crate::state::LauncherPaths;
use crate::util::errors::AnyError;

pub const SERVICE_LOG_FILE_NAME: &str = "tunnel-service.log";

#[async_trait]
pub trait ServiceContainer: Send {
	async fn run_service(
		&mut self,
		log: log::Logger,
		launcher_paths: LauncherPaths,
		shutdown_rx: mpsc::Receiver<ShutdownSignal>,
	) -> Result<(), AnyError>;
}

#[async_trait]
pub trait ServiceManager {
	/// Registers the current executable as a service to run with the given set
	/// of arguments.
	async fn register(&self, exe: PathBuf, args: &[&str]) -> Result<(), AnyError>;

	/// Runs the service using the given handle. The executable *must not* take
	/// any action which may fail prior to calling this to ensure service
	/// states may update.
	async fn run(
		self,
		launcher_paths: LauncherPaths,
		handle: impl 'static + ServiceContainer,
	) -> Result<(), AnyError>;

	/// Show logs from the running service to standard out.
	async fn show_logs(&self) -> Result<(), AnyError>;

	/// Unregisters the current executable as a service.
	async fn unregister(&self) -> Result<(), AnyError>;
}

#[cfg(target_os = "windows")]
pub type ServiceManagerImpl = super::service_windows::WindowsService;

#[cfg(target_os = "linux")]
pub type ServiceManagerImpl = super::service_linux::SystemdService;

#[cfg(target_os = "macos")]
pub type ServiceManagerImpl = super::service_macos::LaunchdService;

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub fn create_service_manager(log: log::Logger, paths: &LauncherPaths) -> ServiceManagerImpl {
	#[cfg(target_os = "macos")]
	{
		super::service_macos::LaunchdService::new(log, paths)
	}
	#[cfg(target_os = "windows")]
	{
		super::service_windows::WindowsService::new(log)
	}
	#[cfg(target_os = "linux")]
	{
		super::service_linux::SystemdService::new(log, paths.clone())
	}
}
