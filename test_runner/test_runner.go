package main

import (
	"fmt"
	"os"
)

type testRunner struct {
	failedTests []string
}

func (runner *testRunner) runTest(test func(), name string) {
	defer func() {
		if err := recover(); err != nil {
			runner.failedTests = append(runner.failedTests, fmt.Sprintf("%s: %v", name, err))
			fmt.Print(" fail\n")
		}
	}()

	fmt.Printf("%s...", name)
	test()
	fmt.Print(" pass\n")
}

func main() {
	runner := &testRunner{}
	/* replace: tests */
	for _, failure := range runner.failedTests {
		fmt.Printf("%s\n", failure)
	}
	if len(runner.failedTests) > 0 {
		os.Exit(1)
	}
}