use rpkg::debversion::VersionRelation;
use rpkg::debversion::{self, DebianVersionNum};

use crate::Packages;
use crate::packages::Dependency;
use std::collections::VecDeque;
use std::collections::HashSet;

impl Packages {
    /// Computes a solution for the transitive dependencies of package_name; when there is a choice A | B | C, 
    /// chooses the first option A. Returns a Vec<i32> of package numbers.
    ///
    /// Note: does not consider which packages are installed.
    pub fn transitive_dep_solution(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let deps : &Vec<Dependency> = &*self.dependencies.get(self.get_package_num(package_name)).unwrap();
        // Create a queue to act as a worklist (pop new work from front, add work to the back)
        let mut worklist: VecDeque<i32> = VecDeque::new();

        // Create hashset such that duplicate dependencies are handled
        let mut dependency_set: HashSet<i32> = HashSet::new(); 

        for dep in deps {
            worklist.push_back(dep[0].package_num);
        }

        while let Some(front) = worklist.pop_front() {
            dependency_set.insert(front);
            let new_deps = self.dependencies.get(&front).unwrap();
            for dep in new_deps {
                if !dependency_set.contains(&dep[0].package_num) {
                    worklist.push_back(dep[0].package_num);
                }
            }
        }

        // Convert hashset back into vector to return
        let dependecy_list: Vec<i32> = dependency_set.into_iter().collect();

        return dependecy_list;
    }

    /// Computes a set of packages that need to be installed to satisfy package_name's deps given the current installed packages.
    /// When a dependency A | B | C is unsatisfied, there are two possible cases:
    ///   (1) there are no versions of A, B, or C installed; pick the alternative with the highest version number (yes, compare apples and oranges).
    ///   (2) at least one of A, B, or C is installed (say A, B), but with the wrong version; of the installed packages (A, B), pick the one with the highest version number.
    pub fn compute_how_to_install(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }
        // implement more sophisticated worklist

        let deps : &Vec<Dependency> = &*self.dependencies.get(self.get_package_num(package_name)).unwrap();
        // Create a queue to act as a worklist (pop new work from front, add work to the back)
        let mut worklist: VecDeque<i32> = VecDeque::new();

        // Create hashset such that duplicate dependencies are handled
        let mut dependencies_to_add: HashSet<i32> = HashSet::new(); 

        for dep in deps {
            match self.handle_dependency(dep) {
                Some(package) => {worklist.push_back(package)}
                None => {}
            }
        }

        while let Some(front) = worklist.pop_front() {
            dependencies_to_add.insert(front);
            let new_deps = self.dependencies.get(&front).unwrap();
            for dep in new_deps {
                match self.handle_dependency(dep) {
                    Some(package) => {
                        worklist.push_back(package);
                    }
                    None => {}
                }
            }
        }

        // Convert hashset back into vector to return
        let dependecy_list: Vec<i32> = dependencies_to_add.into_iter().collect();

        return dependecy_list;

    }
    pub fn handle_dependency(&self, dd:&Dependency) -> Option<i32> {
        match self.dep_is_satisfied_2(dd) {
            Some(package_number) => {
                return None
            }
            None => {
                let installed_incorrect_versions = self.dep_satisfied_by_wrong_version(dd);
                // If the length of packages is 1, we return that 
                if installed_incorrect_versions.len() == 1 {
                    return Some(installed_incorrect_versions[0].0);
                }

                if installed_incorrect_versions.len() > 1 {
                    // CASE: We are picking between installed package that have incorrect versions
                    let mut highest_version_number = installed_incorrect_versions[0].1;
                    let mut selected_package_number = installed_incorrect_versions[0].0;
                    for package in installed_incorrect_versions {
                        let v = self.available_debvers.get(&package.0).unwrap();
                        if debversion::cmp_debversion_with_op(&VersionRelation::StrictlyGreater, &v, highest_version_number) {
                            highest_version_number = package.1;
                            selected_package_number = package.0;
                        }
                    }
                    return Some(selected_package_number);
                } else {
                    // CASE: All alternatives are not installed
                    let mut highest_version_number = self.available_debvers.get(&dd[0].package_num).unwrap();
                    let mut selected_package_number = dd[0].package_num;

                    for package in dd {
                        let v = self.available_debvers.get(&package.package_num).unwrap();

                        if debversion::cmp_debversion_with_op(&VersionRelation::StrictlyGreater, &v, highest_version_number) {
                            highest_version_number = v.clone();
                            selected_package_number = package.package_num;
                        }
                    }
                    return Some(selected_package_number);
                }
            }
        }
    }
}

