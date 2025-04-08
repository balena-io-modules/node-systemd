/**
 * We need to import binaries according to
 * https://nodejs.github.io/node-addon-examples/build-tools/node-pre-gyp/#javascript-updates
 * however, node-pre-gyp does not have typescript declarations so
 * eslint will complain, which is why we add the eslint-disable-line
 * statements
 */
const binary = require('@mapbox/node-pre-gyp'); // eslint-disable-line
const path = require('path'); // eslint-disable-line
const bindingPath = binary.find(
	path.resolve(path.join(__dirname, '../package.json')),
);
const binding = require(bindingPath); // eslint-disable-line

export declare class SystemBus {
	// Needed for typechecking
	private static readonly __id: unique symbol;

	// Do not allow direct instantiation
	// or sub-classing
	private constructor();
}

export declare function system(): Promise<SystemBus>;

// These methods
export declare function unitActiveState(
	bus: SystemBus,
	unitName: string,
): Promise<string>;
export declare function unitPartOf(
	bus: SystemBus,
	unitName: string,
): Promise<string[]>;
export declare function unitStart(
	bus: SystemBus,
	unitName: string,
	mode: string,
): Promise<void>;
export declare function unitStop(
	bus: SystemBus,
	unitName: string,
	mode: string,
): Promise<void>;
export declare function unitRestart(
	bus: SystemBus,
	unitName: string,
	mode: string,
): Promise<void>;
export declare function reboot(
	bus: SystemBus,
	interactive: boolean,
): Promise<void>;
export declare function powerOff(
	bus: SystemBus,
	interactive: boolean,
): Promise<void>;

module.exports = exports = binding;
