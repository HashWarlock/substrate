// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use frame_support::traits::tokens::*;

impl<ClassId, InstanceId> Locker<ClassId, InstanceId> for () {
    /// Check if the asset should be locked and prevent interactions with the asset from executing.
    /// Default will be false if not implemented downstream
    ///
    /// Note: The logic check in this function must be constant time and consistent for benchmarks
    /// to work
    fn is_locked(_class: ClassId, _instance: InstanceId) -> bool {
        false
    }
}
