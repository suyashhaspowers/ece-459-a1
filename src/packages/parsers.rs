use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use itertools::assert_equal;
use regex::Regex;

use crate::Packages;
use crate::packages::RelVersionedPackageNum;

use rpkg::debversion::{self, DebianVersionNum};

const KEYVAL_REGEX : &str = r"(?P<key>(\w|-)+): (?P<value>.+)";
const PKGNAME_AND_VERSION_REGEX : &str = r"(?P<pkg>(\w|\.|\+|-)+)( \((?P<op>(<|=|>)(<|=|>)?) (?P<ver>.*)\))?";

impl Packages {
    /// Loads packages and version numbers from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate value into the installed_debvers map with the parsed version number.
    pub fn parse_installed(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    // do something with ip
                    match kv_regexp.captures(&ip) {
                        None => (),
                        Some(caps) => {
                            let (key, value) = (caps.name("key").unwrap().as_str(), caps.name("value").unwrap().as_str());
                            if key == "Package" {
                                current_package_num = self.get_package_num_inserting(&value);
                            } else if key == "Version" {
                                let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                                self.installed_debvers.insert(current_package_num, debver);
                            }
                        }
                    }
                }
            }
        }
        println!("Packages installed: {}", self.installed_debvers.keys().len());
    }

    /// Loads packages, version numbers, dependencies, and md5sums from a file, calling get_package_num_inserting on the package name
    /// and inserting the appropriate values into the dependencies, md5sum, and available_debvers maps.
    pub fn parse_packages(&mut self, filename: &str) {
        let kv_regexp = Regex::new(KEYVAL_REGEX).unwrap();
        let pkgver_regexp = Regex::new(PKGNAME_AND_VERSION_REGEX).unwrap();

        if let Ok(lines) = read_lines(filename) {
            let mut current_package_num = 0;
            for line in lines {
                if let Ok(ip) = line {
                    match kv_regexp.captures(&ip) {
                        None => (),
                        Some(caps) => {
                            let (key, value) = (caps.name("key").unwrap().as_str(), caps.name("value").unwrap().as_str());
                            if key == "Package" {
                                current_package_num = self.get_package_num_inserting(&value);
                            } else if key == "Version" {
                                let debver = value.trim().parse::<debversion::DebianVersionNum>().unwrap();
                                self.available_debvers.insert(current_package_num, debver);
                            } else if key == "MD5sum" {
                                self.md5sums.insert(current_package_num, value.to_string());
                            } else if key == "Depends" {
                                // Split the string based on commas
                                let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();

                                // Filter out empty strings from the split
                                let non_empty_parts: Vec<&str> = parts.into_iter().filter(|&s| !s.is_empty()).collect();

                                let mut dependency_vect: Vec<Vec<RelVersionedPackageNum>> = Vec::new();

                                for element in &non_empty_parts {
                                    let mut alternatives_vect: Vec<RelVersionedPackageNum> = Vec::new();
                                    // Split the string based on |
                                    let alternatives: Vec<&str> = element.split('|').map(|s| s.trim()).collect();

                                    for alternative in &alternatives {
                                        // Regex on Alternative to parse op, pkg, version
                                        match pkgver_regexp.captures(&alternative) {
                                            None => (),
                                            Some(caps) => {
                                                // Parse name, op, ver to create RelVersionedPackageNum
                                                let package_name = caps.name("pkg").expect("Package name not found in regex match").as_str();
                                                let rel_version = caps.name("op").map(|op| {
                                                    (
                                                        op.as_str().parse::<debversion::VersionRelation>().expect("Error parsing version relation"),
                                                        caps.name("ver").map_or_else(|| "".to_string(), |ver| ver.as_str().to_string()),
                                                    )
                                                });
                                                let package_num = self.get_package_num_inserting(&package_name);
                                                let final_package = RelVersionedPackageNum {
                                                    package_num,
                                                    rel_version,
                                                };
                                                alternatives_vect.push(final_package);
                                            }
                                        }
                                    }

                                    dependency_vect.push(alternatives_vect);
                                }
                                self.dependencies.insert(current_package_num, dependency_vect);
                                
                                // TEST TO SEE OUTPUT OF DEPENDENCIES
                                // println!("******************DEPENDENCIES:");
                                // for alternatives_vect in &dependency_vect {
                                //     println!("****NEW DEPENDENCY****");
                                //     for package in alternatives_vect {
                                //         if alternatives_vect.len() > 1{
                                //             println!("////////////--------------//////-------||||")
                                //         }
                                //         // Print information about each RelVersionedPackageNum
                                //         println!("Package Num: {}", package.package_num);
                                
                                //         if let Some((op, ver)) = &package.rel_version {
                                //             println!("  Version Relation: {}", op);
                                //             println!("  Version: {}", ver);
                                //         } else {
                                //             println!("  Version information not available");
                                //         }
                                //     }
                                // }
                            }
                        }
                    }
                }
            }
        }
        println!("Packages available: {}", self.available_debvers.keys().len());
    }
}


// standard template code downloaded from the Internet somewhere
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
