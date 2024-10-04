# balena-systemd

This module provides some essential functions to interact with [systemd Manager](https://www.freedesktop.org/software/systemd/man/org.freedesktop.systemd1.html) and [systemd-login Manager](https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html) services via D-Bus.

It uses Rust [zbus crate](https://crates.io/crates/zbus) to perform queries to the D-Bus socket and bind results to Node.js using [neon-bindings](https://neon-bindings.com/).

This project has no goals of providing feature parity with the systemd API and new features will be added as-needed. PRs are welcome.

This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).

## Supported features

### ServiceManager

* Manager Object
	- Methods
		- [x] `GetUnit`
		- [x] `StartUnit`
		- [x] `StopUnit`
		- [x] `RestartUnit`
* Unit Object
	- Properties
		- [x] `ActiveState`
		- [x] `PartOf`


**Example**

```
import {ServiceManager, system} from '@balena/systemd';

(async() {
	// This returns a new connection to the bus
	// use singleton() if want to share a connection across
    // your code
	const bus = await system();
	const manager = new ServiceManager(bus);
	const unit = manager.getUnit('openvpn.service');

	// The property needs to be awaited
	const state = await unit.activeState;
	
	console.log('Unit openvpn.service state is', state);

    // Start the service.
    await unit.start();

    console.log('Unit openvpn.service state is now', await unit.activeState);
})();
```

## LoginManager

* Manager Object
	- Methods
		- [x] `Reboot`
		- [x] `PowerOff`

**Example**

```
import {LoginManager, system} from '@balena/systemd';

(async() {
	// This returns a new connection to the bus
	// use singleton() if want to share a connection across
	const bus = await system();
	const manager = new LoginManager(bus);

	// WARNING: this WILL reboot the system!
	await manager.reboot(false);
})();
```

## Installing balena-systemd

Installing the module requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

You can install the project with npm. In the project directory, run:

```sh
$ npm install
```

This fully installs the project, including installing any dependencies and running the build.

## Building balena-systemd

If you have already installed the project and only want to run the build, run:

```sh
$ npm run build
```

This command uses the [cargo-cp-artifact](https://github.com/neon-bindings/cargo-cp-artifact) utility to run the Rust build and copy the built library into `./index.node`. The build produces static binaries. Running this command will also compile the TypeScript sources and store the output under `./build`.

## Running a project that depends on this module

You probably need libstdc

## Run integration tests

Integration tests are run automatically on each PR. To run the full test suite locally, you'll need Docker and docker-compose, and do

```
npm run test:compose
```
