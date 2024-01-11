struct YakGraph {}

// Calculate function dependencies for a package
// This should answer:
//  - What functions are called by other functions
//  - What types are instantiated or referenced by functions
struct YakSourceCodeGraph {}

// Calculate the package dependency graph.
// This should detect cycles.
struct YakPackageDependencyGraph {}
