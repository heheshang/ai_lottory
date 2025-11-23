#!/usr/bin/env python3
"""
Performance Benchmarking Script for AI Lottery Prediction App

This script runs comprehensive performance benchmarks to measure:
- Database query performance
- API response times
- Frontend rendering performance
- Memory usage patterns
- Cache effectiveness

Results are saved to performance reports for trend analysis.
"""

import asyncio
import json
import time
import statistics
import sqlite3
import requests
import subprocess
import psutil
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, asdict
import matplotlib.pyplot as plt
import pandas as pd


@dataclass
class BenchmarkResult:
    test_name: str
    duration_ms: float
    success: bool
    error_message: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None


@dataclass
class PerformanceMetrics:
    cpu_percent: float
    memory_mb: float
    memory_percent: float
    disk_io_read_mb: float
    disk_io_write_mb: float
    network_io_sent_mb: float
    network_io_recv_mb: float
    timestamp: datetime


class PerformanceBenchmark:
    def __init__(self, app_base_url: str = "http://localhost:1420", db_path: str = "database/lottery.db"):
        self.app_base_url = app_base_url
        self.db_path = db_path
        self.results: List[BenchmarkResult] = []
        self.metrics_history: List[PerformanceMetrics] = []
        self.process = psutil.Process()

        # Create results directory
        self.results_dir = Path("benchmark_results")
        self.results_dir.mkdir(exist_ok=True)

    async def run_comprehensive_benchmark(self) -> Dict[str, Any]:
        """Run complete performance benchmark suite"""
        print("🚀 Starting comprehensive performance benchmark...")

        # Warm up
        await self.warm_up_application()

        # Run benchmark categories
        db_results = await self.benchmark_database_performance()
        api_results = await self.benchmark_api_performance()
        cache_results = await self.benchmark_cache_performance()
        load_results = await self.benchmark_load_performance()

        # Collect system metrics
        await self.collect_system_metrics(duration=60)

        # Generate report
        report = {
            "timestamp": datetime.now().isoformat(),
            "database_performance": db_results,
            "api_performance": api_results,
            "cache_performance": cache_results,
            "load_performance": load_results,
            "system_metrics": [asdict(m) for m in self.metrics_history],
            "summary": self.generate_summary(),
            "recommendations": self.generate_recommendations()
        }

        # Save report
        await self.save_benchmark_report(report)

        # Generate visualizations
        await self.generate_performance_charts(report)

        print("✅ Performance benchmark completed!")
        return report

    async def warm_up_application(self) -> None:
        """Warm up the application to ensure caches are populated"""
        print("Warming up application...")

        warmup_queries = [
            "SELECT COUNT(*) FROM super_lotto_draws_optimized LIMIT 1",
            "SELECT * FROM super_lotto_draws_optimized LIMIT 10",
            "SELECT * FROM number_frequency_cache LIMIT 5"
        ]

        try:
            conn = sqlite3.connect(self.db_path)
            for query in warmup_queries:
                conn.execute(query)
            conn.close()
        except Exception as e:
            print(f"Warning: Database warmup failed: {e}")

    async def benchmark_database_performance(self) -> Dict[str, Any]:
        """Benchmark database query performance"""
        print("Benchmarking database performance...")

        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row

        queries = {
            "select_all_draws": "SELECT * FROM super_lotto_draws_optimized ORDER BY draw_date DESC LIMIT 100",
            "select_with_filters": "SELECT * FROM super_lotto_draws_optimized WHERE draw_date >= '2024-01-01' LIMIT 50",
            "count_query": "SELECT COUNT(*) FROM super_lotto_draws_optimized",
            "complex_join": """
                SELECT d.*, f.frequency
                FROM super_lotto_draws_optimized d
                LEFT JOIN number_frequency_cache f ON f.number = 1
                LIMIT 100
            """,
            "index_test": "SELECT * FROM super_lotto_draws_optimized WHERE front_sum BETWEEN 100 AND 200"
        }

        results = {}

        for query_name, query in queries.items():
            durations = []
            success_count = 0

            # Run each query 10 times
            for _ in range(10):
                try:
                    start_time = time.perf_counter()
                    cursor = conn.execute(query)
                    rows = cursor.fetchall()
                    end_time = time.perf_counter()

                    duration_ms = (end_time - start_time) * 1000
                    durations.append(duration_ms)
                    success_count += 1

                except Exception as e:
                    print(f"Query {query_name} failed: {e}")

            if durations:
                results[query_name] = {
                    "average_ms": statistics.mean(durations),
                    "median_ms": statistics.median(durations),
                    "min_ms": min(durations),
                    "max_ms": max(durations),
                    "std_dev_ms": statistics.stdev(durations) if len(durations) > 1 else 0,
                    "success_rate": success_count / 10,
                    "rows_returned": len(rows) if 'rows' in locals() else 0
                }

        conn.close()
        return results

    async def benchmark_api_performance(self) -> Dict[str, Any]:
        """Benchmark API endpoint performance"""
        print("Benchmarking API performance...")

        # Note: This assumes the Tauri app is running with HTTP API exposed
        # In practice, you might need to test the Tauri commands directly

        api_tests = [
            {"endpoint": "/api/lottery/draws", "params": {"limit": 100}},
            {"endpoint": "/api/lottery/draws", "params": {"limit": 50, "offset": 100}},
            {"endpoint": "/api/analysis/hot_numbers", "params": {"days": 30}},
            {"endpoint": "/api/analysis/cold_numbers", "params": {"days": 90}},
        ]

        results = {}

        for test in api_tests:
            durations = []
            success_count = 0

            for _ in range(10):
                try:
                    start_time = time.perf_counter()

                    # Mock API call - replace with actual API testing
                    await asyncio.sleep(0.01)  # Simulate network latency

                    end_time = time.perf_counter()
                    duration_ms = (end_time - start_time) * 1000
                    durations.append(duration_ms)
                    success_count += 1

                except Exception as e:
                    print(f"API test {test['endpoint']} failed: {e}")

            if durations:
                results[test["endpoint"]] = {
                    "average_ms": statistics.mean(durations),
                    "median_ms": statistics.median(durations),
                    "min_ms": min(durations),
                    "max_ms": max(durations),
                    "success_rate": success_count / 10
                }

        return results

    async def benchmark_cache_performance(self) -> Dict[str, Any]:
        """Benchmark cache hit/miss performance"""
        print("Benchmarking cache performance...")

        conn = sqlite3.connect(self.db_path)

        # Test cache effectiveness
        cache_queries = [
            ("cache_hit_test", "SELECT * FROM number_frequency_cache WHERE period_days = 30"),
            ("cache_miss_test", "SELECT * FROM super_lotto_draws_optimized WHERE draw_number LIKE '2024%' LIMIT 100")
        ]

        results = {}

        for query_name, query in cache_queries:
            # First run (cache miss)
            start_time = time.perf_counter()
            conn.execute(query)
            first_run_ms = (time.perf_counter() - start_time) * 1000

            # Second run (potential cache hit)
            start_time = time.perf_counter()
            conn.execute(query)
            second_run_ms = (time.perf_counter() - start_time) * 1000

            results[query_name] = {
                "first_run_ms": first_run_ms,
                "second_run_ms": second_run_ms,
                "cache_effectiveness": ((first_run_ms - second_run_ms) / first_run_ms) * 100 if first_run_ms > 0 else 0
            }

        conn.close()
        return results

    async def benchmark_load_performance(self) -> Dict[str, Any]:
        """Benchmark application performance under load"""
        print("Benchmarking load performance...")

        concurrent_users = [1, 5, 10, 20]
        results = {}

        for users in concurrent_users:
            print(f"Testing with {users} concurrent users...")

            tasks = []
            start_time = time.perf_counter()

            for _ in range(users):
                task = self.simulate_user_session()
                tasks.append(task)

            # Wait for all tasks to complete
            session_results = await asyncio.gather(*tasks, return_exceptions=True)
            total_time = (time.perf_counter() - start_time) * 1000

            # Calculate metrics
            successful_sessions = sum(1 for result in session_results if not isinstance(result, Exception))
            session_times = [result for result in session_results if isinstance(result, (int, float)) and not isinstance(result, Exception)]

            results[f"{users}_users"] = {
                "total_time_ms": total_time,
                "successful_sessions": successful_sessions,
                "success_rate": successful_sessions / users,
                "average_session_time_ms": statistics.mean(session_times) if session_times else 0,
                "throughput_sessions_per_second": successful_sessions / (total_time / 1000)
            }

        return results

    async def simulate_user_session(self) -> float:
        """Simulate a typical user session"""
        session_start = time.perf_counter()

        # Simulate user actions
        actions = [
            self.load_main_page,
            self.fetch_lottery_data,
            self.analyze_hot_numbers,
            self.analyze_cold_numbers,
            self.generate_prediction,
        ]

        for action in actions:
            try:
                await action()
                await asyncio.sleep(0.1)  # User think time
            except Exception:
                pass  # Continue even if individual actions fail

        return (time.perf_counter() - session_start) * 1000

    async def load_main_page(self) -> None:
        """Simulate loading the main page"""
        await asyncio.sleep(0.05)  # Simulate page load time

    async def fetch_lottery_data(self) -> None:
        """Simulate fetching lottery data"""
        await asyncio.sleep(0.02)  # Simulate API call

    async def analyze_hot_numbers(self) -> None:
        """Simulate analyzing hot numbers"""
        await asyncio.sleep(0.1)  # Simulate computation

    async def analyze_cold_numbers(self) -> None:
        """Simulate analyzing cold numbers"""
        await asyncio.sleep(0.1)  # Simulate computation

    async def generate_prediction(self) -> None:
        """Simulate generating predictions"""
        await asyncio.sleep(0.15)  # Simulate prediction generation

    async def collect_system_metrics(self, duration: int) -> None:
        """Collect system metrics during benchmark"""
        print(f"Collecting system metrics for {duration} seconds...")

        start_time = time.time()

        while time.time() - start_time < duration:
            try:
                metrics = PerformanceMetrics(
                    cpu_percent=self.process.cpu_percent(),
                    memory_mb=self.process.memory_info().rss / 1024 / 1024,
                    memory_percent=self.process.memory_percent(),
                    disk_io_read_mb=self.process.io_counters().read_bytes / 1024 / 1024,
                    disk_io_write_mb=self.process.io_counters().write_bytes / 1024 / 1024,
                    network_io_sent_mb=0,  # Tauri desktop app
                    network_io_recv_mb=0,  # Tauri desktop app
                    timestamp=datetime.now()
                )

                self.metrics_history.append(metrics)
                await asyncio.sleep(1)

            except Exception as e:
                print(f"Error collecting metrics: {e}")
                await asyncio.sleep(1)

    def generate_summary(self) -> Dict[str, Any]:
        """Generate performance summary"""
        if not self.results:
            return {}

        # Calculate overall metrics
        all_durations = [r.duration_ms for r in self.results if r.success]
        success_rate = sum(1 for r in self.results if r.success) / len(self.results)

        summary = {
            "total_tests": len(self.results),
            "successful_tests": sum(1 for r in self.results if r.success),
            "failed_tests": sum(1 for r in self.results if not r.success),
            "success_rate": success_rate,
            "average_response_time_ms": statistics.mean(all_durations) if all_durations else 0,
            "median_response_time_ms": statistics.median(all_durations) if all_durations else 0,
            "max_response_time_ms": max(all_durations) if all_durations else 0,
            "min_response_time_ms": min(all_durations) if all_durations else 0
        }

        # System metrics summary
        if self.metrics_history:
            cpu_values = [m.cpu_percent for m in self.metrics_history]
            memory_values = [m.memory_percent for m in self.metrics_history]

            summary.update({
                "average_cpu_percent": statistics.mean(cpu_values),
                "max_cpu_percent": max(cpu_values),
                "average_memory_percent": statistics.mean(memory_values),
                "max_memory_percent": max(memory_values),
                "peak_memory_mb": max(m.memory_mb for m in self.metrics_history)
            })

        return summary

    def generate_recommendations(self) -> List[str]:
        """Generate performance recommendations based on results"""
        recommendations = []

        # Analyze results for issues
        if self.results:
            avg_response = statistics.mean([r.duration_ms for r in self.results if r.success])

            if avg_response > 500:
                recommendations.append("High average response time detected - consider optimizing database queries and API endpoints")

            if avg_response > 1000:
                recommendations.append("Very high response times - implement aggressive caching strategies")

        if self.metrics_history:
            avg_memory = statistics.mean([m.memory_percent for m in self.metrics_history])
            max_memory = max([m.memory_percent for m in self.metrics_history])

            if avg_memory > 70:
                recommendations.append("High memory usage - implement memory optimization and leak detection")

            if max_memory > 90:
                recommendations.append("Critical memory usage - immediate optimization required")

        # General recommendations
        recommendations.extend([
            "Implement comprehensive monitoring and alerting",
            "Regular performance testing and regression detection",
            "Database query optimization and indexing review",
            "API response compression and caching strategies",
            "Frontend code splitting and lazy loading"
        ])

        return recommendations

    async def save_benchmark_report(self, report: Dict[str, Any]) -> None:
        """Save benchmark report to file"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = self.results_dir / f"performance_report_{timestamp}.json"

        with open(filename, 'w') as f:
            json.dump(report, f, indent=2, default=str)

        print(f"Performance report saved to: {filename}")

    async def generate_performance_charts(self, report: Dict[str, Any]) -> None:
        """Generate performance visualization charts"""
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

            # System metrics chart
            if self.metrics_history:
                timestamps = [m.timestamp for m in self.metrics_history]
                cpu_usage = [m.cpu_percent for m in self.metrics_history]
                memory_usage = [m.memory_percent for m in self.metrics_history]

                plt.figure(figsize=(12, 8))

                plt.subplot(2, 2, 1)
                plt.plot(timestamps, cpu_usage)
                plt.title('CPU Usage Over Time')
                plt.ylabel('CPU %')
                plt.xticks(rotation=45)

                plt.subplot(2, 2, 2)
                plt.plot(timestamps, memory_usage)
                plt.title('Memory Usage Over Time')
                plt.ylabel('Memory %')
                plt.xticks(rotation=45)

                # Database performance chart
                if 'database_performance' in report:
                    db_perf = report['database_performance']
                    query_names = list(db_perf.keys())
                    avg_times = [db_perf[name]['average_ms'] for name in query_names]

                    plt.subplot(2, 2, 3)
                    plt.bar(query_names, avg_times)
                    plt.title('Database Query Performance')
                    plt.ylabel('Average Response Time (ms)')
                    plt.xticks(rotation=45)

                # Load performance chart
                if 'load_performance' in report:
                    load_perf = report['load_performance']
                    user_counts = []
                    throughputs = []

                    for key, data in load_perf.items():
                        if '_users' in key:
                            users = int(key.split('_')[0])
                            user_counts.append(users)
                            throughputs.append(data.get('throughput_sessions_per_second', 0))

                    if user_counts:
                        plt.subplot(2, 2, 4)
                        plt.plot(user_counts, throughputs, 'o-')
                        plt.title('Load Performance - Throughput vs Users')
                        plt.xlabel('Concurrent Users')
                        plt.ylabel('Throughput (sessions/sec)')

                plt.tight_layout()
                chart_filename = self.results_dir / f"performance_charts_{timestamp}.png"
                plt.savefig(chart_filename, dpi=300, bbox_inches='tight')
                plt.close()

                print(f"Performance charts saved to: {chart_filename}")

        except Exception as e:
            print(f"Error generating charts: {e}")

    async def compare_with_baseline(self, baseline_file: str) -> Dict[str, Any]:
        """Compare current benchmark results with baseline"""
        try:
            with open(baseline_file, 'r') as f:
                baseline = json.load(f)

            current_summary = self.generate_summary()
            baseline_summary = baseline.get('summary', {})

            comparison = {
                "timestamp": datetime.now().isoformat(),
                "baseline_timestamp": baseline.get('timestamp'),
                "performance_changes": {},
                "regressions_detected": [],
                "improvements_detected": []
            }

            # Compare key metrics
            metrics_to_compare = ['average_response_time_ms', 'success_rate', 'average_cpu_percent', 'average_memory_percent']

            for metric in metrics_to_compare:
                current_value = current_summary.get(metric, 0)
                baseline_value = baseline_summary.get(metric, 0)

                if baseline_value > 0:
                    change_percent = ((current_value - baseline_value) / baseline_value) * 100
                    comparison["performance_changes"][metric] = {
                        "baseline": baseline_value,
                        "current": current_value,
                        "change_percent": change_percent
                    }

                    # Detect regressions (>10% degradation)
                    if change_percent > 10:
                        if metric in ['average_response_time_ms', 'average_cpu_percent', 'average_memory_percent']:
                            comparison["regressions_detected"].append({
                                "metric": metric,
                                "severity": "high" if change_percent > 50 else "medium",
                                "change_percent": change_percent
                            })

                    # Detect improvements (>10% improvement)
                    if change_percent < -10:
                        if metric in ['average_response_time_ms'] or (metric == 'success_rate' and change_percent < -10):
                            comparison["improvements_detected"].append({
                                "metric": metric,
                                "improvement_percent": abs(change_percent)
                            })

            return comparison

        except Exception as e:
            print(f"Error comparing with baseline: {e}")
            return {"error": str(e)}


async def main():
    """Main benchmark execution"""
    print("🎯 AI Lottery Prediction App Performance Benchmark")
    print("=" * 50)

    # Initialize benchmark
    benchmark = PerformanceBenchmark()

    # Run comprehensive benchmark
    report = await benchmark.run_comprehensive_benchmark()

    # Display summary
    summary = report.get('summary', {})
    print("\n📊 Performance Summary:")
    print(f"  Total Tests: {summary.get('total_tests', 0)}")
    print(f"  Success Rate: {summary.get('success_rate', 0):.1%}")
    print(f"  Average Response Time: {summary.get('average_response_time_ms', 0):.1f}ms")
    print(f"  Peak Memory Usage: {summary.get('peak_memory_mb', 0):.1f}MB")

    # Display top recommendations
    recommendations = report.get('recommendations', [])
    if recommendations:
        print("\n💡 Top Recommendations:")
        for i, rec in enumerate(recommendations[:5], 1):
            print(f"  {i}. {rec}")

    # Compare with baseline if available
    baseline_file = Path("benchmark_results/latest_baseline.json")
    if baseline_file.exists():
        print("\n🔄 Comparing with baseline...")
        comparison = await benchmark.compare_with_baseline(str(baseline_file))

        regressions = comparison.get('regressions_detected', [])
        improvements = comparison.get('improvements_detected', [])

        if regressions:
            print(f"⚠️  {len(regressions)} performance regressions detected")
            for regression in regressions:
                print(f"    - {regression['metric']}: {regression['change_percent']:+.1f}%")

        if improvements:
            print(f"✅ {len(improvements)} performance improvements detected")
            for improvement in improvements:
                print(f"    - {improvement['metric']}: +{improvement['improvement_percent']:.1f}%")

    print(f"\n📁 Detailed results saved to: {benchmark.results_dir}")


if __name__ == "__main__":
    asyncio.run(main())