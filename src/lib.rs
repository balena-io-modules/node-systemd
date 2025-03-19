use anyhow::Context as AnyhowContext;
use neon::prelude::*;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;
use tokio::time::{self, Duration};
use zbus::export::futures_util::StreamExt;
use zbus::export::futures_util::TryFutureExt;
use zbus::zvariant::OwnedObjectPath;
use zbus::{proxy, Connection};

// Return a global tokio runtime or create one if it doesn't exist.
// Throws a JavaScript exception if the `Runtime` fails to create.
fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
pub trait ServiceManager {
    #[zbus(object = "Unit")]
    fn get_unit(&self, unit: &str) -> zbus::Result<Unit>;

    #[zbus(object = "Job")]
    fn start_unit(&self, unit: &str, mode: &str) -> zbus::Result<Job>;

    #[zbus(object = "Job")]
    fn stop_unit(&self, unit: &str, mode: &str) -> zbus::Result<Job>;

    #[zbus(object = "Job")]
    fn restart_unit(&self, unit: &str, mode: &str) -> zbus::Result<Job>;
}

#[proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.systemd1.Job"
)]
pub trait Job {}

#[proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.systemd1.Unit"
)]
pub trait Unit {
    #[zbus(property)]
    fn active_state(&mut self) -> zbus::Result<String>;

    #[zbus(property)]
    fn sub_state(&mut self) -> zbus::Result<String>;

    #[zbus(property)]
    fn part_of(&mut self) -> zbus::Result<Vec<String>>;

    #[zbus(property)]
    fn exec_main_status(&self) -> zbus::Result<i32>;

    #[zbus(object = "Job")]
    fn start(&self, mode: &str) -> zbus::Result<Job>;
}

#[proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.systemd1.Service"
)]
trait Service {
    #[zbus(property)]
    fn exec_main_status(&self) -> zbus::Result<i32>;
}

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait LoginManager {
    fn reboot(&self, interactive: bool) -> zbus::Result<()>;
    fn power_off(&self, interactive: bool) -> zbus::Result<()>;
}

// This is the object that will get exposed to
// the javascript API
struct System {
    connection: Connection,
}

// Needed to be able to box the System struct
impl Finalize for System {}

/// Create a new connection to the system bus
fn system(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let rt = runtime(&mut cx)?;
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    rt.spawn(async move {
        // Create the connection in a background thread
        // we await the result here, but we only unwrap it inside the promise
        // to avoid unhandle promise rejections
        let connection = Connection::system().await;
        deferred.settle_with(&channel, move |mut cx| {
            let connection = connection.or_else(|e| {
                cx.throw_error(format!("Failed to connect to D-Bus system socket: {}", e))
            })?;

            let system = System { connection };

            Ok(cx.boxed(system))
        });
    });

    Ok(promise)
}

// Here we implement the functions that will get exposed
// to javascript
impl System {
    /// Get the active state of a provided unit
    fn unit_active_state(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let unit_name = cx.argument::<JsString>(1)?.value(&mut cx);
        let channel = cx.channel();

        // We need to clone the connection because we are going to move it into
        // the spawned task. Zbus documentation reports that this is a very cheap
        // operation and it seems that this is the way to share connections
        // between threads
        // https://docs.rs/zbus/3.0.0/zbus/struct.Connection.html
        let connection = system.connection.clone();

        // It is important to be careful not to perform failable actions after
        // creating the promise to avoid an unhandled rejection.
        let (deferred, promise) = cx.promise();

        // Run operations on a background thread
        rt.spawn(async move {
            // We chain the promises with `and_then` so we can get the error
            // to reject the promise in the
            // settle_with block
            let state = ServiceManagerProxy::new(&connection)
                .and_then(|manager| async move {
                    let mut unit = manager.get_unit(&unit_name).await?;
                    unit.active_state().await
                })
                .await;

            // Settle the promise from the result of a closure. JavaScript exceptions
            // will be converted to a Promise rejection.
            //
            // This closure will execute on the JavaScript main thread. It should be
            // limited to converting Rust types to JavaScript values. Expensive operations
            // should be performed outside of it.
            deferred.settle_with(&channel, move |mut cx| {
                // Convert a `zbus::Error` to a JavaScript exception
                let state = state.or_else(|err| cx.throw_error(err.to_string()))?;
                Ok(cx.string(state))
            });
        });

        Ok(promise)
    }

    fn unit_part_of(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let unit_name = cx.argument::<JsString>(1)?.value(&mut cx);
        let channel = cx.channel();

        let connection = system.connection.clone();
        let (deferred, promise) = cx.promise();

        rt.spawn(async move {
            let state = ServiceManagerProxy::new(&connection)
                .and_then(|manager| async move {
                    let mut unit = manager.get_unit(&unit_name).await?;
                    unit.part_of().await
                })
                .await;

            // Settle the promise from the result of a closure. JavaScript exceptions
            // will be converted to a Promise rejection.
            //
            // This closure will execute on the JavaScript main thread. It should be
            // limited to converting Rust types to JavaScript values. Expensive operations
            // should be performed outside of it.
            deferred.settle_with(&channel, move |mut cx| {
                // Convert a `zbus::Error` to a JavaScript exception
                let state = state.or_else(|err| cx.throw_error(err.to_string()))?;

                let res = cx.empty_array();
                for (i, unit) in state.iter().enumerate() {
                    let unit = cx.string(unit);
                    res.set(&mut cx, i as u32, unit)?;
                }

                Ok(res)
            });
        });

        Ok(promise)
    }

    fn unit_start(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let unit_name = cx.argument::<JsString>(1)?.value(&mut cx);
        let mode = cx.argument::<JsString>(2)?.value(&mut cx);
        let channel = cx.channel();

        let connection = system.connection.clone();
        let (deferred, promise) = cx.promise();

        // Run operations on a background thread
        rt.spawn(async move {
            let result = ServiceManagerProxy::new(&connection)
                .and_then(|manager| async move { manager.start_unit(&unit_name, &mode).await })
                .await;

            deferred.settle_with(&channel, move |mut cx| {
                result.or_else(|err| cx.throw_error(err.to_string()))?;
                Ok(cx.undefined())
            });
        });

        Ok(promise)
    }

    fn unit_start_and_wait(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let service_name = cx.argument::<JsString>(1)?.value(&mut cx);
        let wait_interval = cx.argument::<JsNumber>(2)?.value(&mut cx) as u64;
        let mode = cx.argument::<JsString>(3)?.value(&mut cx);
        let channel = cx.channel();

        let connection = system.connection.clone();
        let (deferred, promise) = cx.promise();

        // Start and wait functionality is defined in a separate function, which returns
        // a Result, so that it is easier to propagate error conditions, e.g. unit does
        // not exist, etc.
        async fn start_and_wait_unit(
            service_name: &str,
            connection: Connection,
            mode: &str,
            wait_interval: u64,
        ) -> anyhow::Result<(String, i32, String)> {
            let unit_path_str = service_to_unit_path(service_name);

            let unit_path =
                OwnedObjectPath::try_from(unit_path_str.clone()).with_context(|| {
                    format!("Cannot convert unit name `{service_name}` to service path")
                })?;

            let mut unit = UnitProxy::builder(&connection)
                .path(unit_path.clone())
                .with_context(|| format!("Cannot set unit path from {unit_path_str}"))?
                .build()
                .await
                .with_context(|| format!("Cannot build unit proxy for {unit_path_str}"))?;

            let mut active_state = unit
                .active_state()
                .await
                .context("Failed to get unit active state")?;

            // Create a D-Bus stream that watches for service active state changes
            let mut stream = unit.receive_active_state_changed().await;

            unit.start(mode)
                .await
                .with_context(|| format!("Failed to start unit {unit_path_str}"))?;

            let wait_duration = Duration::from_secs(wait_interval);

            // Either wait for next active state change event or stop if timeout is reached
            while let Ok(result) = time::timeout(wait_duration, stream.next()).await {
                match result {
                    Some(value) => {
                        active_state = value
                            .get()
                            .await
                            .context("Failed to read unit active state")?;
                    }
                    None => {
                        // Active state change stream has ended.
                        break;
                    }
                }
            }

            let service = ServiceProxy::builder(&connection)
                .path(unit_path.clone())
                .with_context(|| format!("Cannot set service path from {unit_path_str}"))?
                .build()
                .await
                .with_context(|| format!("Cannot build service proxy for {unit_path_str}"))?;

            let exec_status = service.exec_main_status().await.with_context(|| {
                format!("Failed to get main process status for {unit_path_str}")
            })?;

            let sub_state = unit
                .sub_state()
                .await
                .with_context(|| format!("Failed to get unit sub state for {unit_path_str}"))?;

            Ok((active_state, exec_status, sub_state))
        }

        // Run operations on a background thread
        rt.spawn(async move {
            let result = start_and_wait_unit(&service_name, connection, &mode, wait_interval).await;

            deferred.settle_with(&channel, move |mut cx| {
                let (active_state, exec_status, sub_state) =
                    result.or_else(|err| cx.throw_error(err.to_string()))?;

                let obj = cx.empty_object();

                let state = cx.string(active_state);
                obj.set(&mut cx, "state", state)
                    .expect("Cannot set object 'state' property");

                // Use `code` instead of sub-state in order to use systemctl naming
                let code = cx.string(sub_state);
                obj.set(&mut cx, "code", code)
                    .expect("Cannot set object 'code' property");

                // Use `status` instead of exec-status in order to use systemctl naming
                let status = cx.number(exec_status);
                obj.set(&mut cx, "status", status)
                    .expect("Cannot set object 'status' property");

                // Returns an object containing state, code and status,
                // e.g. `{ state: 'failed', code: 'failed', status: 26 }`
                Ok(obj)
            });
        });

        Ok(promise)
    }

    fn unit_stop(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let unit_name = cx.argument::<JsString>(1)?.value(&mut cx);
        let mode = cx.argument::<JsString>(2)?.value(&mut cx);
        let channel = cx.channel();

        let connection = system.connection.clone();
        let (deferred, promise) = cx.promise();

        // Run operations on a background thread
        rt.spawn(async move {
            let result = ServiceManagerProxy::new(&connection)
                .and_then(|manager| async move { manager.stop_unit(&unit_name, &mode).await })
                .await;

            deferred.settle_with(&channel, move |mut cx| {
                result.or_else(|err| cx.throw_error(err.to_string()))?;
                Ok(cx.undefined())
            });
        });

        Ok(promise)
    }

    fn unit_restart(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let unit_name = cx.argument::<JsString>(1)?.value(&mut cx);
        let mode = cx.argument::<JsString>(2)?.value(&mut cx);
        let channel = cx.channel();

        let connection = system.connection.clone();
        let (deferred, promise) = cx.promise();

        // Run operations on a background thread
        rt.spawn(async move {
            let result = ServiceManagerProxy::new(&connection)
                .and_then(|manager| async move { manager.restart_unit(&unit_name, &mode).await })
                .await;

            deferred.settle_with(&channel, move |mut cx| {
                result.or_else(|err| cx.throw_error(err.to_string()))?;
                Ok(cx.undefined())
            });
        });

        Ok(promise)
    }

    fn reboot(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let interactive = cx.argument::<JsBoolean>(1)?.value(&mut cx);
        let channel = cx.channel();

        let connection = system.connection.clone();
        let (deferred, promise) = cx.promise();

        // Run operations on a background thread
        rt.spawn(async move {
            let result = LoginManagerProxy::new(&connection)
                .and_then(|manager| async move { manager.reboot(interactive).await })
                .await;

            deferred.settle_with(&channel, move |mut cx| {
                result.or_else(|err| cx.throw_error(err.to_string()))?;
                Ok(cx.undefined())
            });
        });

        Ok(promise)
    }

    fn power_off(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let rt = runtime(&mut cx)?;
        let system = cx.argument::<JsBox<System>>(0)?;
        let interactive = cx.argument::<JsBoolean>(1)?.value(&mut cx);
        let channel = cx.channel();

        let connection = system.connection.clone();
        let (deferred, promise) = cx.promise();

        // Run operations on a background thread
        rt.spawn(async move {
            let result = LoginManagerProxy::new(&connection)
                .and_then(|manager| async move { manager.power_off(interactive).await })
                .await;

            deferred.settle_with(&channel, move |mut cx| {
                result.or_else(|err| cx.throw_error(err.to_string()))?;
                Ok(cx.undefined())
            });
        });

        Ok(promise)
    }
}

fn service_to_unit_path(service_name: &str) -> String {
    // Some symbols that may exist in service names (e.g. `.` or `-`) has to be encoded
    // when transformed into D-Bus paths.
    format!(
        "/org/freedesktop/systemd1/unit/{}",
        service_name
            .replace('.', "_2e")
            .replace('-', "_2d")
            .replace(':', "_3a")
    )
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("system", system)?;
    cx.export_function("unitActiveState", System::unit_active_state)?;
    cx.export_function("unitPartOf", System::unit_part_of)?;
    cx.export_function("unitStart", System::unit_start)?;
    cx.export_function("unitStartAndWait", System::unit_start_and_wait)?;
    cx.export_function("unitStop", System::unit_stop)?;
    cx.export_function("unitRestart", System::unit_restart)?;
    cx.export_function("reboot", System::reboot)?;
    cx.export_function("powerOff", System::power_off)?;
    Ok(())
}
