# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

#####################################################################
# Providers

SwiftToolchainInfo = provider(fields = [
    "architecture",
    "can_toolchain_emit_obj_c_header_textually",  # bool
    "uncompiled_swift_sdk_modules_deps",  # {str.type: dependency} Expose deps of uncompiled Swift SDK modules.
    "uncompiled_clang_sdk_modules_deps",  # {str.type: dependency} Expose deps of uncompiled Clang SDK modules.
    "compiler_flags",
    "compiler",
    "prefix_serialized_debugging_options",  # bool
    "resource_dir",  # "artifact",
    "sdk_path",
    "swift_stdlib_tool_flags",
    "swift_stdlib_tool",
])

# A provider that represents a non-yet-compiled SDK (Swift or Clang) module,
# and doesn't contain any artifacts because Swift toolchain isn't resolved yet.
SdkUncompiledModuleInfo = provider(fields = [
    "name",  # A name of a module with `.swift`/`.clang` suffix.
    "module_name",  # A real name of a module, without distinguishing suffixes.
    "is_framework",  # This is mostly needed for the generated Swift module map file.
    "is_swiftmodule",  # If True then represents a swiftinterface, otherwise Clang's modulemap.
    "partial_cmd",  # Partial arguments, required to compile a particular SDK module.
    "input_relative_path",  # A relative prefixed path to a textual swiftinterface/modulemap file within an SDK.
    "deps",  # ["dependency"]
])

WrappedSdkCompiledModuleInfo = provider(fields = [
    "tset",  # A tset that contains SdkCompiledModuleInfo itself and its transitive deps
])

# A provider that represents an already-compiled SDK (Swift or Clang) module.
SdkCompiledModuleInfo = provider(fields = [
    "name",  # A name of a module with `.swift`/`.clang` suffix.
    "module_name",  # A real name of a module, without distinguishing suffixes.
    "is_swiftmodule",  # If True then contains a compiled swiftmodule, otherwise Clang's pcm.
    "is_framework",
    "output_artifact",  # Compiled artifact either swiftmodule or pcm.
    "input_relative_path",
])

SdkSwiftOverlayInfo = provider(fields = [
    "overlays",  # {str.type: [str.type]} A mapping providing a list of overlay module names for each underlying module
])
