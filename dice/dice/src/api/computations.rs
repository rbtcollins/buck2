/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::future::Future;

use allocative::Allocative;
use dupe::Dupe;
use gazebo::prelude::*;
use more_futures::cancellation::CancellationContext;

use crate::api::data::DiceData;
use crate::api::error::DiceResult;
use crate::api::key::Key;
use crate::api::opaque::OpaqueValue;
use crate::api::transaction::DiceTransaction;
use crate::api::user_data::UserComputationData;
use crate::ctx::DiceComputationsImpl;
use crate::UserCycleDetectorGuard;

/// The context for computations to register themselves, and request for additional dependencies.
/// The dependencies accessed are tracked for caching via the `DiceCtx`.
///
/// The computations are registered onto `DiceComputations` via implementing traits for the
/// `DiceComputation`.
///
/// The context is valid only for the duration of the computation of a single key, and cannot be
/// owned.
#[derive(Allocative, Dupe, Clone)]
#[repr(transparent)]
pub struct DiceComputations(pub(crate) DiceComputationsImpl);

fn _test_computations_sync_send() {
    fn _assert_sync_send<T: Sync + Send>() {}
    _assert_sync_send::<DiceComputations>();
}

impl DiceComputations {
    /// Gets all the result of of the given computation key.
    /// recorded as dependencies of the current computation for which this
    /// context is for.
    pub fn compute<'a, K>(
        &'a self,
        key: &'a K,
    ) -> impl Future<Output = DiceResult<<K as Key>::Value>> + 'a
    where
        K: Key,
    {
        self.0.compute(key)
    }

    /// same as `compute` but for a multiple keys. The returned results will be in order of the
    /// keys given
    pub fn compute_many<'a, K>(
        &'a self,
        keys: &[&'a K],
    ) -> Vec<impl Future<Output = DiceResult<<K as Key>::Value>> + 'a>
    where
        K: Key,
    {
        keys.map(|k| self.compute(*k))
    }

    /// Compute "opaque" value where the value is only accessible via projections.
    /// Projections allow accessing derived results from the "opaque" value,
    /// where the dependency of reading a projection is the projection value rather
    /// than the entire opaque value.
    pub fn compute_opaque<'b, 'a: 'b, K>(
        &'a self,
        key: &'b K,
    ) -> impl Future<Output = DiceResult<OpaqueValue<'a, K>>> + 'b
    where
        K: Key,
    {
        self.0.compute_opaque(key)
    }

    /// temporarily here while we figure out why dice isn't paralleling computations so that we can
    /// use this in tokio spawn. otherwise, this shouldn't be here so that we don't need to clone
    /// the Arc, which makes lifetimes weird.
    pub fn temporary_spawn<F, FUT, R>(&self, f: F) -> impl Future<Output = R> + Send + 'static
    where
        F: FnOnce(DiceTransaction, &CancellationContext) -> FUT + Send + 'static,
        FUT: Future<Output = R> + Send,
        R: Send + 'static,
    {
        self.0.temporary_spawn(async move |ctx| {
            f(
                DiceTransaction(DiceComputations(ctx)),
                &CancellationContext::todo(),
            )
            .await
        })
    }

    /// Data that is static per the entire lifetime of Dice. These data are initialized at the
    /// time that Dice is initialized via the constructor.
    pub fn global_data(&self) -> &DiceData {
        self.0.global_data()
    }

    /// Data that is static for the lifetime of the current request context. This lifetime is
    /// the lifetime of the top-level `DiceComputation` used for all requests.
    /// The data is also specific to each request context, so multiple concurrent requests can
    /// each have their own individual data.
    pub fn per_transaction_data(&self) -> &UserComputationData {
        self.0.per_transaction_data()
    }

    /// Gets the current cycle guard if its set. If it's set but a different type, an error will be returned.
    pub fn cycle_guard<T: UserCycleDetectorGuard>(&self) -> DiceResult<Option<&T>> {
        self.0.cycle_guard()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use allocative::Allocative;
    use derive_more::Display;
    use dupe::Dupe;
    use indexmap::indexset;

    use crate::api::cycles::DetectCycles;
    use crate::api::error::DiceErrorImpl;
    use crate::api::storage_type::StorageType;
    use crate::api::user_data::UserComputationData;
    use crate::legacy::ctx::ComputationData;
    use crate::legacy::cycles::RequestedKey;
    use crate::legacy::incremental::graph::storage_properties::StorageProperties;

    #[derive(Clone, Dupe, Display, Debug, PartialEq, Eq, Hash, Allocative)]
    struct K(usize);
    impl StorageProperties for K {
        type Key = K;

        type Value = ();

        fn key_type_name() -> &'static str {
            unreachable!()
        }

        fn to_key_any(key: &Self::Key) -> &dyn std::any::Any {
            key
        }

        fn storage_type(&self) -> StorageType {
            unreachable!()
        }

        fn equality(&self, _x: &Self::Value, _y: &Self::Value) -> bool {
            unreachable!()
        }

        fn validity(&self, _x: &Self::Value) -> bool {
            unreachable!()
        }
    }

    #[test]
    fn cycle_detection_when_no_cycles() -> anyhow::Result<()> {
        let ctx = ComputationData::new(UserComputationData::new(), DetectCycles::Enabled);
        let ctx = ctx.subrequest::<K>(&K(1))?;
        let ctx = ctx.subrequest::<K>(&K(2))?;
        let ctx = ctx.subrequest::<K>(&K(3))?;
        let _ctx = ctx.subrequest::<K>(&K(4))?;

        Ok(())
    }

    #[test]
    fn cycle_detection_when_cycles() -> anyhow::Result<()> {
        let ctx = ComputationData::new(UserComputationData::new(), DetectCycles::Enabled);
        let ctx = ctx.subrequest::<K>(&K(1))?;
        let ctx = ctx.subrequest::<K>(&K(2))?;
        let ctx = ctx.subrequest::<K>(&K(3))?;
        let ctx = ctx.subrequest::<K>(&K(4))?;
        match ctx.subrequest::<K>(&K(1)) {
            Ok(_) => {
                panic!("should have cycle error")
            }
            Err(e) => match &*e.0 {
                DiceErrorImpl::Cycle {
                    trigger,
                    cyclic_keys,
                } => {
                    assert!(
                        (**trigger).get_key_equality() == K(1).get_key_equality(),
                        "expected trigger key to be `{}` but was `{}`",
                        K(1),
                        trigger
                    );
                    assert_eq!(
                        cyclic_keys,
                        &indexset![
                            Arc::new(K(1)) as Arc<dyn RequestedKey>,
                            Arc::new(K(2)) as Arc<dyn RequestedKey>,
                            Arc::new(K(3)) as Arc<dyn RequestedKey>,
                            Arc::new(K(4)) as Arc<dyn RequestedKey>
                        ]
                    )
                }
                _ => {
                    panic!("wrong error type")
                }
            },
        }

        Ok(())
    }
}
