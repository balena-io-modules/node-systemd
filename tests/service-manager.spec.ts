import { expect } from './chai';
import { singleton, ServiceManager } from '../lib';

describe('ServiceManager', () => {
	describe('Unit', () => {
		it('activeState can be queried', async () => {
			const bus = await singleton();
			const manager = new ServiceManager(bus);
			await expect(manager.getUnit('dummy.service').activeState).to.not.be
				.rejected;
		});

		it('activeState can be queried in parallel', async () => {
			const bus = await singleton();
			const manager = new ServiceManager(bus);
			// TODO: how can we test that the call is indeed not blocking the
			// main thread?
			await expect(
				Promise.all([
					manager.getUnit('dummy.service').activeState,
					manager.getUnit('dummy.service').activeState,
				]),
			).to.not.be.rejected;
		});

		it('activeState starts as "active"', async () => {
			const bus = await singleton();
			const manager = new ServiceManager(bus);
			await expect(
				manager.getUnit('dummy.service').activeState,
			).to.eventually.equal('active');
		});

		it('partOf can be queried', async () => {
			const bus = await singleton();
			const manager = new ServiceManager(bus);
			await expect(
				manager.getUnit('dummy.service').partOf,
			).to.eventually.deep.equal([]);
		});
	});

	describe('ServiceManager', () => {
		it('allows to start unit', async () => {
			const bus = await singleton();
			const unit = new ServiceManager(bus).getUnit('dummy.service');

			await expect(unit.start('fail')).to.not.be.rejected;
			await expect(unit.activeState).to.eventually.equal('active');
		});

		it('allows to stop unit', async () => {
			const bus = await singleton();
			const unit = new ServiceManager(bus).getUnit('dummy.service');

			await expect(unit.stop('fail')).to.not.be.rejected;
			await expect(unit.activeState).to.eventually.equal('inactive');
		});

		it('allows to restart unit', async () => {
			const bus = await singleton();
			const unit = new ServiceManager(bus).getUnit('dummy.service');

			await expect(unit.restart('fail')).to.not.be.rejected;
			await expect(unit.activeState).to.eventually.equal('active');
			await expect(unit.stop('fail')).to.not.be.rejected;
			await expect(unit.activeState).to.eventually.equal('inactive');
			await expect(unit.restart('fail')).to.not.be.rejected;
			await expect(unit.activeState).to.eventually.equal('active');
		});
	});
});
