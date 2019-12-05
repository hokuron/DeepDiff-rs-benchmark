//
//  DeepDiff+Rust.swift
//  Benchmark
//
//  Created by hokuron on 2019/12/01.
//  Copyright © 2019 Ryo Aoyama. All rights reserved.
//

import Foundation

@inlinable
func diffStrings(old: [UnsafePointer<CChar>?], new: [UnsafePointer<CChar>?]) {
    diffWithString(old, Int32(old.count), new, Int32(new.count))
}

@inlinable
func coercion(_ string: UnsafePointer<CChar>) -> UnsafePointer<CChar>? {
    return string
}
