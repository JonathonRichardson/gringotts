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

var exec = function(command, config) {
  config = config || {};
  return child_process.execSync(command, _.extend(execConfig, config)).toString();
}

describe("cargo test", function() {
  it ("should pass all tests", function(done) {
    child_process.exec("cargo test 2>&1", function(error, stdout, stderr) {
      expect(error).toBeNull();
      done();
    })
  });
});

describe("dbctl", function() {
  var dbctl = function(command, file, extraArgs, config) {
    config = config || config;
    return exec('../target/debug/dbctl ' + command + ' --database-file ' + file + ' ' + extraArgs, config);
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
      var output = exec("xxd " + testdbfile + " | grep -E '^0000100' | tr -d '\n'");
      expect(output).toMatch(/: 424c 0000 0000 16/);
    });

    it("should not create the file twice", function() {
      var output = dbctl("create", testdbfile);

      expect(output).not.toMatch(/Successfully created/i);
      expect(output).toMatch(/already exists/i);
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

  describe("get/set", function() {
    beforeAll(function() {
      dbctl('create', testdbfile);
    });

    afterAll(function() {
      fs.unlinkSync(testdbfile);
    });

    it("should store and retrieve information", function() {
      var key1 = "new key";
      var val1 = "new value";
      dbctl("set", testdbfile, key1, {input: val1});
      var output1 = dbctl("get", testdbfile, key1);
      expect(output1).toBe(val1);

      var key2 = "newkey2";
      var val2 = "asdf\0garbage";
      dbctl("set", testdbfile, key2, {input: val2});
      var output2 = dbctl("get", testdbfile, key2);

      output1 = dbctl("get", testdbfile, key1);
      expect(output1).toBe(val1);
      expect(output2).toBe(val2);
    });

    it("should store and retrieve multiple levels of information", function() {
      var key1 = "path/to/data";
      var val1 = "path/to/more/data";
      dbctl("set", testdbfile, key1, {input: val1});
      var output1 = dbctl("get", testdbfile, key1);
      expect(output1).toBe(val1);

      var key2 = "newkey2";
      var val2 = "asdf\0garbage";
      dbctl("set", testdbfile, key2, {input: val2});
      var output2 = dbctl("get", testdbfile, key2);

      output1 = dbctl("get", testdbfile, key1);
      expect(output1).toBe(val1);
      expect(output2).toBe(val2);
    });

    it("should store and retrieve many nodes at one level", function() {
      var val = "DATA"
      for(var i = 0; i < 1000; i++) {
        var key = "records/" + i;
        dbctl("set", testdbfile, key, {input: val});
      }

      var output = dbctl("get", testdbfile, "records/999");
      expect(output).toBe(val);

      output = dbctl("get", testdbfile, "records/300");
      expect(output).toBe(val);
    });
  });
});
