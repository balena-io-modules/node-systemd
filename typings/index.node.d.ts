declare module "*index.node" {
	class SystemBus {
		// Needed for typechecking
		private __id: unique symbol

		// Do not allow direct instantiation
		// or sub-classing
		private constructor();
	};

	function system(): Promise<SystemBus>;

	// These methods
	function unitActiveState(bus: SystemBus, unitName: string): Promise<string>;
	function unitPartOf(bus: SystemBus, unitName: string): Promise<string[]>;
	function unitStart(bus: SystemBus, unitName: string, mode: string): Promise<void>;
	function unitStartAndWait(bus: SystemBus, unitName: string, wait: number, mode: string): Promise<object>;
	function unitStop(bus: SystemBus, unitName: string, mode: string): Promise<void>;
	function unitRestart(bus: SystemBus, unitName: string, mode: string): Promise<void>;
	function reboot(bus: SystemBus, interactive: boolean): Promise<void>;
	function powerOff(bus: SystemBus, interactive: boolean): Promise<void>;
}
