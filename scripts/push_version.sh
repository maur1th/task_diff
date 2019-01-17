#!/bin/bash -ex

if [ -n "$TRAVIS_TAG" ];
then
    sed -i "s/^version \{0,1\}=.*/version = \"${TRAVIS_TAG:1}\"/" Cargo.toml
    git config --local user.name "Travis CI"
    git config --local user.email "travis@travis-ci.org"
    git add Cargo.toml
    git commit --allow-empty -m "Bump version to $TRAVIS_TAG [skip ci]"
    git remote set-url origin https://${GITHUB_TOKEN}@github.com/maur1th/task_diff.git
    git pull --rebase origin master
    git push origin HEAD:master
fi
