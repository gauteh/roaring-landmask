#!/usr/bin/env python

import setuptools
from setuptools_rust import RustExtension, Binding

setuptools.setup(
    name='roaring-landmask',
    version='0.1.0',
    description='A fast and limited-memory structure with a landmask based on GSHHG for determing whether a point on Earth is on land or in the ocean',
    author='Gaute Hope',
    author_email='eg@gaute.vetsj.com',
    url='https://github.com/gauteh/roaring-landmask',
    packages=setuptools.find_packages(),
    include_package_data=False,
    setup_requires=['setuptools_scm'],
    rust_extensions=[
        RustExtension("roaring_landmask.roaring_landmask",
                      "Cargo.toml",
                      binding=Binding.PyO3,
                      features=["extension-module"],
                      debug=False)
    ])
