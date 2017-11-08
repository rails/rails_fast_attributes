require "bundler/gem_tasks"
require "rspec/core/rake_task"
require 'helix_runtime/build_task'

RSpec::Core::RakeTask.new(:spec)
HelixRuntime::BuildTask.new

task :default => :spec

task :spec => [:build]
