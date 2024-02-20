use rpkg::debversion::{self, DebianVersionNum};
use crate::Packages;
use crate::packages::Dependency;

impl Packages {
    /// Gets the dependencies of package_name, and prints out whether they are satisfied (and by which library/version) or not.
    pub fn deps_available(&mut self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }
        println!("Package {}:", package_name);

        let package_num = self.get_package_num_inserting(&package_name);
        match self.dependencies.get(&package_num) {
            Some(dependencies) => {
                for dependency in dependencies {
                    println!("- dependency {:?}", self.dep2str(dependency));
                    match self.dep_is_satisfied(dependency) {
                        Some(string) => {
                            println!("{}", string);
                        }
                        None => {
                            println!("-> not satisfied");
                        }
                    }
                }
            }
            None => println!("There are no associated dependencies.")
        }
    }

    /// Returns Some(package) which satisfies dependency dd, or None if not satisfied.
    pub fn dep_is_satisfied(&self, dd:&Dependency) -> Option<String> {
        for package in dd {
            match &package.rel_version {
                Some((op, version_string)) => {
                    let v = version_string.parse::<debversion::DebianVersionNum>().unwrap();
                    match self.installed_debvers.get(&package.package_num) {
                        Some(iv) => {
                            if debversion::cmp_debversion_with_op(&op, &iv, &v) {
                                let package_name = self.get_package_name(package.package_num);
                                return Some(format!("+ {} satisfied by installed version {}", package_name, iv));
                            } else {
                                return None
                            }
                        }
                        None => ()
                    }
                }
                None => ()
            }
            match self.installed_debvers.get(&package.package_num) {
                Some(iv) => {
                    let package_name = self.get_package_name(package.package_num);
                    return Some(format!("+ {} satisfied by installed version {}", package_name, iv));
                }
                None => ()
            }

        }
        return None;
    }

    /// Returns Some(package<i32>) which satisfies dependency dd, or None if not satisfied.
    pub fn dep_is_satisfied_2(&self, dd:&Dependency) -> Option<i32> {
        for package in dd {
            match &package.rel_version {
                Some((op, version_string)) => {
                    let v = version_string.parse::<debversion::DebianVersionNum>().unwrap();
                    match self.installed_debvers.get(&package.package_num) {
                        Some(iv) => {
                            if debversion::cmp_debversion_with_op(&op, &iv, &v) {
                                return Some(package.package_num);
                            } else {
                                return None
                            }
                        }
                        None => ()
                    }
                }
                None => ()
            }
            match self.installed_debvers.get(&package.package_num) {
                Some(iv) => {
                    return Some(package.package_num);
                }
                None => ()
            }

        }
        return None;
    }

    /// Returns a Vec of packages which would satisfy dependency dd but for the version.
    /// Used by the how-to-install command, which calls compute_how_to_install().
    pub fn dep_satisfied_by_wrong_version(&self, dd:&Dependency) -> Vec<(i32,&DebianVersionNum)> {
        assert! (self.dep_is_satisfied(dd).is_none());
        let mut result = vec![];
        // another loop on dd

        for package in dd {
            match &package.rel_version {
                Some((op, version_string)) => {
                    let v = version_string.parse::<debversion::DebianVersionNum>().unwrap();
                    match self.installed_debvers.get(&package.package_num) {
                        Some(iv) => {
                            if !debversion::cmp_debversion_with_op(&op, &iv, &v) {
                                result.push((package.package_num, iv));
                            }
                        }
                        None => ()
                    }
                }
                None => ()
            }
        }
        return result;
    }
}

