package main
import (
	"bufio"
	"fmt"
	"os"
	"runtime"
	"strconv"
	"sync"
	"time"
	"math"
)
//primality check using the 6k +/- 1 rule
func isPrime(n int) bool {
	if n <=1 { return false }
	if n <=3 { return true }
	if n%2 == 0 || n%3 == 0 { return false }

	limit := int(math.Sqrt(float64(n)))
	for i := 5; i <= limit; i += 6 {
		if n%i == 0 || n%(i+2) == 0 {
			return false
		}
	}
	return true
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run primeCount.go numbers.txt")
		os.Exit(1)
		}
	//file reading (not timed)
	file, err := os.Open(os.Args[1])
	scanner := bufio.NewScanner(file)
	var numbers []int
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" { continue }
		if val, err := strconv.Atoi(line); err == nil {
			numbers = append(numbers, val)
			}
		}
	file.Close()

	//single-threaded approach
	start := time.Now()
	singleCount := 0
	for _, n := range numbers {
		if isPrime(n) { singleCount++ }
		}
	singleDuration := time.Since(start)

	//multi-threaded approach
	numCores := 2
	chunkSize := (len(numbers) + numCores - 1) / numCores
	var wg sync.WaitGroup
	results := make(chan int, numCores)

	startMulti := time.Now()
	for i := 0; i<numCores; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			count := 0
			end := (i+1) * chunkSize
			if end > len(numbers) { end=len(numbers) }
			for _, n := range numbers[i*chunkSize : end] {
				if isPrime(n) { count++ }
				}
			results <- count
			}(i)
		}
	go func() {
		wg.Wait()
		close(results)
	}()

	multiCount := 0
	for c := range results { multiCount += c }
	multiDuration := time.Since(startMulti)

	//output
	fmt.Printf("File: %s\n", os.Args[1])
	fmt.Printf("[Single-Threaded]\nPrimes found: %d\nTime: %.2f ms\n",
			  singleCount, float64(singleDuration.Nanoseconds()) / 1e6)
	fmt.Printf("[Multi-Threaded]\nPrimes found: %d\nTime: %.2f ms\n",
			  multiCount, float64(multiDuration.Nanoseconds()) / 1e6)
	fmt.Printf("Speedup: %.2fx\n", float64(singleDuration)/float64(multiDuration))
}
