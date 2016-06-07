var gulp = require('gulp');
var child_process = require("child_process");
var jasmine = require('gulp-jasmine');
var JasmineConsoleReporter = require('jasmine-console-reporter');
var reporter = new JasmineConsoleReporter();

gulp.task('default', ['build']);

gulp.task('test', ['build'], function() {
  gulp.src('test/test.js').pipe(jasmine({
      reporter: reporter,
      config: {
        "spec_dir": "test",
        "spec_files": [
          "**.js"
        ],
        "helpers": [
          "helpers/**/*.js"
        ],
        "stopSpecOnExpectationFailure": false,
        "random": false
      }
    }))
});

gulp.task('build', function() {
  child_process.execSync('cargo build')
});
