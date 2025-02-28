/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use buck2_core::fs::project_rel_path::ProjectRelativePathBuf;

/// Structured format for an "offline archive manifest", which contains information
/// necessary to perform a fully offline build of a particular target.
///
/// This manifest is generated by running:
///   `buck2 debug io-trace export-manifest`
#[derive(Debug, Clone, serde::Serialize)]
pub struct OfflineArchiveManifest {
    /// The repository revision this archive was generated from.
    pub repo_revision: Option<String>,
    /// List of project-relative paths that are required to perform a build.
    pub paths: Vec<ProjectRelativePathBuf>,
}
