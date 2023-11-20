import {
	SystemBus,
	unitActiveState,
	unitPartOf,
	unitStart,
	unitStop,
	unitRestart,
	powerOff,
	reboot,
	system,
} from './systemd-bindings';

export { system, SystemBus } from './systemd-bindings';

/**
 * Convenience method to return a singleton instance of the system bus.
 *
 * Use this instead of system() if you want to avoid creating
 * multiple connections
 */
export const singleton = (() => {
	let bus: SystemBus | null = null;
	return async function () {
		if (!bus) {
			bus = await system();
		}
		return bus;
	};
})();

/**
 * See: https://www.freedesktop.org/wiki/Software/systemd/dbus/
 */
export class ServiceManager {
	constructor(readonly bus: SystemBus) {}

	getUnit(name: string) {
		return new Unit(this.bus, name);
	}
}

/**
 * Systemd Job mode
 *
 * From: https://www.freedesktop.org/wiki/Software/systemd/dbus/
 *
 * > The mode needs to be one of replace, fail, isolate, ignore-dependencies, ignore-requirements. If "replace" the call will start the unit and its dependencies, possibly replacing already queued jobs that conflict with this. If "fail" the call will start the unit and its dependencies, but will fail if this would change an already queued job. If "isolate" the call will start the unit in question and terminate all units that aren't dependencies of it. If "ignore-dependencies" it will start a unit but ignore all its dependencies. If "ignore-requirements" it will start a unit but only ignore the requirement dependencies. It is not recommended to make use of the latter two options. Returns the newly created job object.
 */
export type JobMode = 'replace' | 'fail' | 'isolate' | 'ignore-dependencies';

export class Unit {
	constructor(
		readonly bus: SystemBus,
		readonly name: string,
	) {}

	/**
	 * Return the unit state
	 *
	 * From: https://www.freedesktop.org/wiki/Software/systemd/dbus/
	 *
	 * > ActiveState contains a state value that reflects whether the unit is currently active or not. The following states are currently defined: active, reloading, inactive, failed, activating, deactivating. active indicates that unit is active (obviously...). reloading indicates that the unit is active and currently reloading its configuration. inactive indicates that it is inactive and the previous run was successful or no previous run has taken place yet. failed indicates that it is inactive and the previous run was not successful (more information about the reason for this is available on the unit type specific interfaces, for example for services in the Result property, see below). activating indicates that the unit has previously been inactive but is currently in the process of entering an active state. Conversely deactivating indicates that the unit is currently in the process of deactivation.
	 */
	get activeState(): Promise<string> {
		return unitActiveState(this.bus, this.name);
	}

	/**
	 * Return the list of units this unit is part of, i.e. the units that when restarted
	 * will also trigger a restart of `this` unit.
	 *
	 * https://www.freedesktop.org/software/systemd/man/systemd.unit.html#PartOf=
	 */
	get partOf(): Promise<string[]> {
		return unitPartOf(this.bus, this.name);
	}

	/**
	 * Enqueues a start job and possibly dependent jobs.
	 *
	 *
	 * From: https://www.freedesktop.org/wiki/Software/systemd/dbus/
	 *
	 * > The mode needs to be one of replace, fail, isolate, ignore-dependencies, ignore-requirements. If "replace" the call will start the unit and its dependencies, possibly replacing already queued jobs that conflict with this. If "fail" the call will start the unit and its dependencies, but will fail if this would change an already queued job. If "isolate" the call will start the unit in question and terminate all units that aren't dependencies of it. If "ignore-dependencies" it will start a unit but ignore all its dependencies. If "ignore-requirements" it will start a unit but only ignore the requirement dependencies. It is not recommended to make use of the latter two options. Returns the newly created job object.
	 */
	async start(mode: JobMode = 'fail'): Promise<void> {
		await unitStart(this.bus, this.name, mode);
	}

	/**
	 * Enqueues a stop job and possibly dependent jobs.
	 *
	 * This defaults to `fail` mode
	 *
	 * @see: https://www.freedesktop.org/wiki/Software/systemd/dbus/
	 * @see Unit.star
	 */
	async stop(mode: JobMode = 'fail'): Promise<void> {
		await unitStop(this.bus, this.name, mode);
	}

	/**
	 * Enqueues a stop job and possibly dependent jobs.
	 *
	 * This defaults to `fail` mode
	 *
	 * See: https://www.freedesktop.org/wiki/Software/systemd/dbus/
	 */
	async restart(mode: JobMode = 'fail'): Promise<void> {
		await unitRestart(this.bus, this.name, mode);
	}
}

/**
 * See https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html
 */
export class LoginManager {
	constructor(readonly bus: SystemBus) {}

	/**
	 * Reboot the system.
	 *
	 * This defaults to not asking for user confirmation.
	 *
	 * From: https://www.freedesktop.org/wiki/Software/systemd/dbus/
	 */
	async reboot(interactive = false): Promise<void> {
		await reboot(this.bus, interactive);
	}

	/**
	 * Power off the system
	 *
	 * This defaults to not asking for user confirmation.
	 */
	async powerOff(interactive = false): Promise<void> {
		await powerOff(this.bus, interactive);
	}
}
