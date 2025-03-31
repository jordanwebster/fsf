class TestRunner {
    constructor() {
        this.failedTests = [];
    }

    runTest(test, name) {
        process.stdout.write(`${name}...`);

        try {
            test();
            process.stdout.write(" pass\n");
        } catch (err) {
            this.failedTests.push(`${name}: ${err.message || err}`);
            process.stdout.write(" fail\n");
        }
    }
}

function main() {
    const runner = new TestRunner();

    /* replace: tests */

    if (runner.failedTests.length > 0) {
        runner.failedTests.forEach(failure => {
            console.log(failure);
        });

        process.exit(1);
    }
}

main()
