var fs = require("fs");
var path = require("path");
var child_process = require("child_process");

var cwd = process.cwd();
var test_dir = path.join(cwd, "test_files");
var testdbfile = path.join(test_dir, "test.db")

describe("dbctl", function() {
  var dbctl = function(command, file) {
    return child_process.execSync('../target/debug/dbctl ' + command + ' ' + file, {
      cwd: test_dir
    })
  };

  beforeAll(function() {
    fs.mkdirSync(test_dir);
  });

  afterAll(function() {
    fs.rmdirSync(test_dir);
  })

  it("should be able to create the db file", function() {
    dbctl('create', testdbfile);

    var fileStat = fs.statSync(testdbfile); // This will throw if it doesn't exist
    fs.unlinkSync(testdbfile);
  });
});
