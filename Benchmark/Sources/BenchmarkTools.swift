import DifferenceKit
import Differentiator
import IGListKit
import DeepDiff

extension String: Differentiable {}

extension String: IdentifiableType {
    @inlinable
    public var identity: String {
        return self
    }
}

struct BenchmarkData {
    var source: [String]
    var target: [String]
    var deleteRange: CountableRange<Int>
    var insertRange: CountableRange<Int>
    var shuffleRange: CountableRange<Int>

    init(count: Int, deleteRange: CountableRange<Int>, insertRange: CountableRange<Int>, shuffleRange: CountableRange<Int>) {
        source = (0..<count).map { _ in UUID().uuidString }
        target = source
        self.deleteRange = deleteRange
        self.insertRange = insertRange
        self.shuffleRange = shuffleRange

        target.removeSubrange(deleteRange)
        target.insert(contentsOf: insertRange.map { _ in UUID().uuidString }, at: insertRange.lowerBound)
        target[shuffleRange].shuffle()
    }
}

struct Benchmark {
    var name: String
    var prepare: (BenchmarkData) -> () -> Void

    func measure(with data: BenchmarkData) -> CFAbsoluteTime {
        let action = prepare(data)
        let start = CFAbsoluteTimeGetCurrent()
        action()
        let end = CFAbsoluteTimeGetCurrent()
        return end - start
    }
}

struct BenchmarkRunner {
    var benchmarks: [Benchmark]

    init(_ benchmarks: Benchmark...) {
        self.benchmarks = benchmarks
    }

    func run(with data: BenchmarkData) {
        let benchmarks = self.benchmarks
        let sourceCount = String.localizedStringWithFormat("%d", data.source.count)
        let deleteCount = String.localizedStringWithFormat("%d", data.deleteRange.count)
        let insertCount = String.localizedStringWithFormat("%d", data.insertRange.count)
        let shuffleCount = String.localizedStringWithFormat("%d", data.shuffleRange.count)

        let maxLength = benchmarks.lazy
            .map { $0.name.count }
            .max() ?? 0

        let empty = String(repeating: " ", count: maxLength)
        let timeTitle = "Time(sec)".padding(toLength: maxLength, withPad: " ", startingAt: 0)
        let leftAlignSpacer = ":" + String(repeating: "-", count: maxLength - 1)
        let rightAlignSpacer = String(repeating: "-", count: maxLength - 1) + ":"

        print("#### - From \(sourceCount) elements to \(deleteCount) deleted, \(insertCount) inserted and \(shuffleCount) shuffled")
        print()
        print("""
            |\(empty)|\(timeTitle)|
            |\(leftAlignSpacer)|\(rightAlignSpacer)|
            """)

        var results = ContiguousArray<CFAbsoluteTime?>(repeating: nil, count: benchmarks.count)
        let group = DispatchGroup()
        let queue = DispatchQueue(label: "Measure benchmark queue", attributes: .concurrent)

        for (offset, benchmark) in benchmarks.enumerated() {
            group.enter()

            queue.async(group: group) {
                let first = benchmark.measure(with: data)
                let second = benchmark.measure(with: data)
                let third = benchmark.measure(with: data)
                results[offset] = min(first, second, third)
                group.leave()
            }
        }

        group.wait()

        for (offset, benchmark) in benchmarks.enumerated() {
            guard let result = results[offset] else {
                fatalError("Measuring was not works correctly.")
            }

            let paddingName = benchmark.name.padding(toLength: maxLength, withPad: " ", startingAt: 0)
            let paddingTime = String(format: "`%.4f`", result).padding(toLength: maxLength, withPad: " ", startingAt: 0)

            print("|\(paddingName)|", terminator: "")
            print("\(paddingTime)|")
        }

        print()
    }
}
