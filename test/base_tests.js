var fs = require("fs");
var path = require("path");
var child_process = require("child_process");
var _ = require("underscore");

var cwd = process.cwd();
var test_dir = path.join(cwd, "test_files");
var testdbfile = path.join(test_dir, "test.db")

var execConfig = {
  cwd: test_dir
};

var exec = function(command) {
  return child_process.execSync(command, execConfig).toString();
}

describe("dbctl", function() {
  var dbctl = function(command, file) {
    return exec('../target/debug/dbctl ' + command + ' --database-file ' + file);
  };

  beforeAll(function() {
    fs.mkdirSync(test_dir);
  });

  afterAll(function() {
    fs.rmdirSync(test_dir);
  });


  describe("generally", function() {
    var checkCommandForPanic = function(command, file) {
      var output;
      try {
        output = exec(command);
      }
      catch (e) {
        throw "dbctl panicked";
      }
      expect(output).not.toMatch(/thread .* panicked/i);

      return output;
    };

    it("should handle bad command names", function() {
      checkCommandForPanic('../target/debug/dbctl nfo --database-file ' + testdbfile);
    });

    it("should handle not getting a filename", function() {
      checkCommandForPanic('../target/debug/dbctl nfo --database-fil ' + testdbfile);
      checkCommandForPanic('../target/debug/dbctl nfo --database-file');
      checkCommandForPanic('../target/debug/dbctl nfo ' + testdbfile);
    });
  })

  describe("create", function() {
    beforeAll(function() {
      dbctl('create', testdbfile);
    });

    afterAll(function() {
      fs.unlinkSync(testdbfile);
    });

    it("should be able to create the db file", function() {
      var fileStat = fs.statSync(testdbfile); // This will throw if it doesn't exist
    });

    it("should start with the magic string", function() {
      var magic_string = exec('head -c 65 ' + testdbfile);
      expect(magic_string).toBe("GringottsDBFile - https://github.com/JonathonRichardson/gringotts");
    });

    it("should set the first block's header line", function() {
      var output = exec("xxd " + testdbfile + " | grep -E '^00001' | tr -d '\n'");
      expect(output).toMatch(/: 0000 0000 1600/);
    });

    xit("should not create the file twice", function() {
      var output = dbctl("create", testdbfile);

      expect(output).not.toMatch(/Successfully created/i);
    });
  });

  describe("info", function() {
    beforeAll(function() {
      dbctl('create', testdbfile);
    });

    afterAll(function() {
      fs.unlinkSync(testdbfile);
    });

    it("should display the correct initial information.", function() {
      var output = dbctl("info", testdbfile);
      var unexecuted_expects = 3;

      _.each(output.split(/\n/), function(line) {
        var key = line.split(/:\s+/)[0];
        var val = line.split(/:\s+/)[1];

        if (key.match(/block size/i)) {
          expect(val).toBe("4kb");
          unexecuted_expects--;
        }
        else if (key.match(/version/i)) {
          expect(val).toBe("0.0.1");
          unexecuted_expects--;
        }
        else if (key.match(/number of blocks/i)) {
          expect(val).toBe("1");
          unexecuted_expects--;
        }
      });

      expect(unexecuted_expects).toBe(0);
    });
  });
});
