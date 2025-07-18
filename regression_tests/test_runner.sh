#!/bin/bash

# Compiler Regression Test Script
# Usage: ./run_tests.sh [test_name] [--overwrite]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Configuration
COMPILER_BUILD_CMD="cargo build --manifest-path ../compiler/Cargo.toml --target-dir ./bin"
COMPILER_PATH="./bin/debug/compiler"
OUTPUT_EXTENSION=".out"

# Global variables
PASSED_TESTS=()
FAILED_TESTS=()
OVERWRITE_MODE=false
SPECIFIC_TEST=""

# Function to show usage
show_usage() {
    echo "Usage: $0 [test_name] [--overwrite]"
    echo ""
    echo "Options:"
    echo "  test_name    Run only the specified test directory"
    echo "  --overwrite  Overwrite .out files with current compiler output"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 my_test           # Run only my_test directory"
    echo "  $0 --overwrite       # Run all tests and update .out files"
    echo "  $0 my_test --overwrite # Run my_test and update my_test.out"
}

# Function to build the compiler
build_compiler() {
    echo -e "${BLUE}Building compiler...${NC}"
    if ! $COMPILER_BUILD_CMD; then
        echo -e "${RED}Failed to build compiler!${NC}"
        exit 1
    fi

    if [ ! -f "$COMPILER_PATH" ]; then
        echo -e "${RED}Compiler binary not found at $COMPILER_PATH${NC}"
        exit 1
    fi

    echo -e "${GREEN}Compiler built successfully${NC}"
    echo ""
}

# Function to check if a path should be ignored based on .gitignore
should_ignore() {
    local path="$1"
    local gitignore_file=".gitignore"

    # If no .gitignore file exists, don't ignore anything
    if [ ! -f "$gitignore_file" ]; then
        return 1
    fi

    # Read .gitignore and check each pattern
    while IFS= read -r pattern || [ -n "$pattern" ]; do
        # Skip empty lines and comments
        if [[ -z "$pattern" ]] || [[ "$pattern" =~ ^[[:space:]]*# ]]; then
            continue
        fi

        # Remove leading/trailing whitespace
        pattern=$(echo "$pattern" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

        # Skip empty patterns after trimming
        if [[ -z "$pattern" ]]; then
            continue
        fi

        # Handle negation patterns (starting with !)
        if [[ "$pattern" =~ ^! ]]; then
            pattern="${pattern#!}"
            # If path matches a negation pattern, it should NOT be ignored
            if [[ "$path" == "$pattern" ]] || [[ "$path/" == "$pattern" ]] || [[ "$path" == "${pattern%/}" ]]; then
                return 1
            fi
        else
            # Check if path matches the pattern
            # Handle directory patterns (ending with /)
            if [[ "$pattern" =~ /$ ]]; then
                pattern="${pattern%/}"
                if [[ "$path" == "$pattern" ]]; then
                    return 0
                fi
            else
                # Handle file/directory patterns
                if [[ "$path" == "$pattern" ]] || [[ "$path/" == "$pattern" ]] || [[ "$path" == "${pattern%/}" ]]; then
                    return 0
                fi
            fi

            # Handle glob patterns with fnmatch-style matching
            case "$path" in
                $pattern) return 0 ;;
            esac
        fi
    done < "$gitignore_file"

    return 1
}

# Function to find all test directories
find_test_directories() {
    local directories=""

    # Find all directories (excluding . and ..)
    for dir in */; do
        # Remove trailing slash
        dir_name="${dir%/}"

        # Skip if directory doesn't exist (in case of no matches)
        if [ ! -d "$dir_name" ]; then
            continue
        fi

        # Check if this directory should be ignored
        if ! should_ignore "$dir_name"; then
            directories="$directories$dir_name"$'\n'
        fi
    done

    # Sort and remove empty lines
    echo "$directories" | grep -v '^$' | sort
}

# Function to run a single test
run_test() {
    local test_name="$1"
    local test_dir="$test_name"
    local expected_file="${test_name}${OUTPUT_EXTENSION}"
    local temp_output=$(mktemp)

    echo -e "${BLUE}Running test: $test_name${NC}"

    # Check if test directory exists
    if [ ! -d "$test_dir" ]; then
        echo -e "${RED}Test directory $test_dir not found!${NC}"
        rm -f "$temp_output"
        return 1
    fi

    # Run the compiler and capture both stdout and stderr
    if ! "$COMPILER_PATH" run "$test_dir" > "$temp_output" 2>&1; then
        # If compiler fails, still capture the output for comparison
        "$COMPILER_PATH" run "$test_dir" > "$temp_output" 2>&1 || true
    fi

    # If overwrite mode, update the expected output file
    if [ "$OVERWRITE_MODE" = true ]; then
        cp "$temp_output" "$expected_file"
        echo -e "${YELLOW}Updated $expected_file${NC}"
        rm -f "$temp_output"
        return 0
    fi

    # Check if expected output file exists
    if [ ! -f "$expected_file" ]; then
        echo -e "${RED}Expected output file $expected_file not found!${NC}"
        echo -e "${YELLOW}Actual output:${NC}"
        cat "$temp_output"
        echo -e "${YELLOW}Consider running with --overwrite to create the expected output file${NC}"
        rm -f "$temp_output"
        return 1
    fi

    # Compare outputs
    if diff -q "$expected_file" "$temp_output" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        PASSED_TESTS+=("$test_name")
        rm -f "$temp_output"
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        echo -e "${YELLOW}Differences:${NC}"

        # Try to use a nice diff tool with color support
        if command -v colordiff > /dev/null 2>&1; then
            diff -u "$expected_file" "$temp_output" | colordiff
        elif command -v diff > /dev/null 2>&1 && diff --help 2>&1 | grep -q -- --color; then
            diff -u --color=always "$expected_file" "$temp_output"
        else
            echo -e "${BLUE}Expected output:${NC}"
            cat "$expected_file"
            echo -e "${BLUE}Actual output:${NC}"
            cat "$temp_output"
            echo -e "${BLUE}Raw diff:${NC}"
            diff -u "$expected_file" "$temp_output" || true
        fi

        FAILED_TESTS+=("$test_name")
        rm -f "$temp_output"
        return 1
    fi
}

# Function to print summary
print_summary() {
    echo ""
    echo "=========================================="
    echo -e "${BLUE}TEST SUMMARY${NC}"
    echo "=========================================="

    local total_tests=$((${#PASSED_TESTS[@]} + ${#FAILED_TESTS[@]}))

    echo -e "${GREEN}Passed: ${#PASSED_TESTS[@]}${NC}"
    if [ ${#PASSED_TESTS[@]} -gt 0 ]; then
        for test in "${PASSED_TESTS[@]}"; do
            echo -e "  ${GREEN}✓${NC} $test"
        done
    fi

    echo -e "${RED}Failed: ${#FAILED_TESTS[@]}${NC}"
    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        for test in "${FAILED_TESTS[@]}"; do
            echo -e "  ${RED}✗${NC} $test"
        done
    fi

    echo "=========================================="
    echo -e "Total tests: $total_tests"

    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        echo -e "${RED}Some tests failed!${NC}"
        exit 1
    else
        echo -e "${GREEN}All tests passed!${NC}"
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --overwrite)
            OVERWRITE_MODE=true
            shift
            ;;
        --help|-h)
            show_usage
            exit 0
            ;;
        -*)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            if [ -z "$SPECIFIC_TEST" ]; then
                SPECIFIC_TEST="$1"
            else
                echo "Multiple test names specified"
                show_usage
                exit 1
            fi
            shift
            ;;
    esac
done

# Main execution
echo -e "${BLUE}Compiler Regression Test Runner${NC}"
echo "======================================"

# Build the compiler
build_compiler

# Run tests
if [ -n "$SPECIFIC_TEST" ]; then
    # Run specific test
    echo -e "${BLUE}Running specific test: $SPECIFIC_TEST${NC}"
    echo ""
    run_test "$SPECIFIC_TEST"
else
    # Run all tests
    echo -e "${BLUE}Running all tests...${NC}"
    echo ""

    test_directories=$(find_test_directories)

    if [ -z "$test_directories" ]; then
        echo -e "${YELLOW}No test directories found${NC}"
        exit 0
    fi

    for test_name in $test_directories; do
        run_test "$test_name"
        echo ""
    done
fi

# Print summary (only if not in overwrite mode for a specific test)
if [ "$OVERWRITE_MODE" = false ] || [ -z "$SPECIFIC_TEST" ]; then
    print_summary
fi