#!/usr/bin/env python3
"""
Integration Tests for MR.DarkPromth
Tests both Staging and Production environments
"""

import requests
import json
import time
import sys
from typing import Dict, List, Tuple

class IntegrationTestSuite:
    def __init__(self):
        self.production_base = "http://localhost:8080"
        self.staging_base = "http://localhost:8081"
        self.staging_nginx = "http://localhost:8082"
        self.results = []
        
    def log_test(self, test_name: str, passed: bool, details: str = ""):
        """Log test result"""
        status = "✅ PASS" if passed else "❌ FAIL"
        result = {
            "test": test_name,
            "status": status,
            "details": details,
            "timestamp": time.time()
        }
        self.results.append(result)
        print(f"{status} {test_name}")
        if details:
            print(f"    {details}")
    
    def test_endpoint(self, url: str, expected_status: int = 200) -> Tuple[bool, Dict]:
        """Test HTTP endpoint"""
        try:
            response = requests.get(url, timeout=10)
            return (response.status_code == expected_status, {
                "status_code": response.status_code,
                "response_time": response.elapsed.total_seconds(),
                "content_length": len(response.content)
            })
        except Exception as e:
            return (False, {"error": str(e)})
    
    def test_api_health(self) -> bool:
        """Test API health endpoints"""
        print("\n=== Testing API Health Endpoints ===")
        
        # Production API
        prod_success, prod_details = self.test_endpoint(f"{self.production_base}/health")
        self.log_test("Production API Health", prod_success, f"Status: {prod_details.get('status_code', 'N/A')}")
        
        # Staging API Direct
        staging_success, staging_details = self.test_endpoint(f"{self.staging_base}/health")
        self.log_test("Staging API Health (Direct)", staging_success, f"Status: {staging_details.get('status_code', 'N/A')}")
        
        # Staging API via Nginx
        staging_nginx_success, staging_nginx_details = self.test_endpoint(f"{self.staging_nginx}/api/status")
        self.log_test("Staging API Health (via Nginx)", staging_nginx_success, f"Status: {staging_nginx_details.get('status_code', 'N/A')}")
        
        return prod_success and staging_success and staging_nginx_success
    
    def test_api_status(self) -> bool:
        """Test API status endpoints"""
        print("\n=== Testing API Status Endpoints ===")
        
        # Production Status
        prod_success, prod_details = self.test_endpoint(f"{self.production_base}/api/status")
        if prod_success:
            try:
                response = requests.get(f"{self.production_base}/api/status")
                status_data = response.json()
                self.log_test("Production API Status", True, f"Environment: {status_data.get('environment', 'N/A')}")
            except:
                self.log_test("Production API Status", False, "Invalid JSON response")
        else:
            self.log_test("Production API Status", False, f"Status: {prod_details.get('status_code', 'N/A')}")
        
        # Staging Status
        staging_success, staging_details = self.test_endpoint(f"{self.staging_base}/api/status")
        if staging_success:
            try:
                response = requests.get(f"{self.staging_base}/api/status")
                status_data = response.json()
                self.log_test("Staging API Status", True, f"Environment: {status_data.get('environment', 'N/A')}")
            except:
                self.log_test("Staging API Status", False, "Invalid JSON response")
        else:
            self.log_test("Staging API Status", False, f"Status: {staging_details.get('status_code', 'N/A')}")
        
        return prod_success and staging_success
    
    def test_database_connectivity(self) -> bool:
        """Test database connectivity"""
        print("\n=== Testing Database Connectivity ===")
        
        # Production DB
        prod_success, prod_details = self.test_endpoint(f"{self.production_base}/api/test/db")
        if prod_success:
            try:
                response = requests.get(f"{self.production_base}/api/test/db")
                db_data = response.json()
                self.log_test("Production Database", True, f"Status: {db_data.get('database_test', 'N/A')}")
            except:
                self.log_test("Production Database", False, "Invalid JSON response")
        else:
            self.log_test("Production Database", False, f"Status: {prod_details.get('status_code', 'N/A')}")
        
        # Staging DB
        staging_success, staging_details = self.test_endpoint(f"{self.staging_base}/api/test/db")
        if staging_success:
            try:
                response = requests.get(f"{self.staging_base}/api/test/db")
                db_data = response.json()
                self.log_test("Staging Database", True, f"Status: {db_data.get('database_test', 'N/A')}")
            except:
                self.log_test("Staging Database", False, "Invalid JSON response")
        else:
            self.log_test("Staging Database", False, f"Status: {staging_details.get('status_code', 'N/A')}")
        
        return prod_success and staging_success
    
    def test_redis_connectivity(self) -> bool:
        """Test Redis connectivity"""
        print("\n=== Testing Redis Connectivity ===")
        
        # Production Redis
        prod_success, prod_details = self.test_endpoint(f"{self.production_base}/api/test/redis")
        if prod_success:
            try:
                response = requests.get(f"{self.production_base}/api/test/redis")
                redis_data = response.json()
                self.log_test("Production Redis", True, f"Status: {redis_data.get('redis_test', 'N/A')}")
            except:
                self.log_test("Production Redis", False, "Invalid JSON response")
        else:
            self.log_test("Production Redis", False, f"Status: {prod_details.get('status_code', 'N/A')}")
        
        # Staging Redis
        staging_success, staging_details = self.test_endpoint(f"{self.staging_base}/api/test/redis")
        if staging_success:
            try:
                response = requests.get(f"{self.staging_base}/api/test/redis")
                redis_data = response.json()
                self.log_test("Staging Redis", True, f"Status: {redis_data.get('redis_test', 'N/A')}")
            except:
                self.log_test("Staging Redis", False, "Invalid JSON response")
        else:
            self.log_test("Staging Redis", False, f"Status: {staging_details.get('status_code', 'N/A')}")
        
        return prod_success and staging_success
    
    def test_metrics_endpoints(self) -> bool:
        """Test Prometheus metrics endpoints"""
        print("\n=== Testing Metrics Endpoints ===")
        
        # Production Metrics
        prod_success, prod_details = self.test_endpoint(f"{self.production_base}/metrics")
        if prod_success:
            content = requests.get(f"{self.production_base}/metrics").text
            metrics_count = content.count('# HELP')
            self.log_test("Production Metrics", True, f"Metrics found: {metrics_count}")
        else:
            self.log_test("Production Metrics", False, f"Status: {prod_details.get('status_code', 'N/A')}")
        
        # Staging doesn't have metrics (simpler version)
        self.log_test("Staging Metrics", True, "Not implemented in staging (expected)")
        
        return prod_success
    
    def test_environment_isolation(self) -> bool:
        """Test that staging and production are isolated"""
        print("\n=== Testing Environment Isolation ===")
        
        try:
            # Get status from both environments
            prod_response = requests.get(f"{self.production_base}/api/status")
            staging_response = requests.get(f"{self.staging_base}/api/status")
            
            prod_data = prod_response.json()
            staging_data = staging_response.json()
            
            # Check environments are different
            prod_env = prod_data.get('environment', '')
            staging_env = staging_data.get('environment', '')
            
            isolation_ok = (prod_env != staging_env and 
                          'production' in prod_env.lower() and 
                          'staging' in staging_env.lower())
            
            self.log_test("Environment Isolation", isolation_ok, 
                         f"Production: {prod_env}, Staging: {staging_env}")
            
            return isolation_ok
        except Exception as e:
            self.log_test("Environment Isolation", False, f"Error: {str(e)}")
            return False
    
    def test_response_times(self) -> bool:
        """Test response times are acceptable"""
        print("\n=== Testing Response Times ===")
        
        # Test production
        start_time = time.time()
        prod_success, prod_details = self.test_endpoint(f"{self.production_base}/health")
        prod_time = time.time() - start_time
        
        # Test staging
        start_time = time.time()
        staging_success, staging_details = self.test_endpoint(f"{self.staging_base}/health")
        staging_time = time.time() - start_time
        
        # Check if response times are acceptable (< 1 second)
        prod_time_ok = prod_time < 1.0
        staging_time_ok = staging_time < 1.0
        
        self.log_test("Production Response Time", prod_time_ok, f"{prod_time:.3f}s")
        self.log_test("Staging Response Time", staging_time_ok, f"{staging_time:.3f}s")
        
        return prod_time_ok and staging_time_ok
    
    def run_all_tests(self) -> Dict:
        """Run all integration tests"""
        print("🚀 Starting MR.DarkPromth Integration Tests")
        print("=" * 50)
        
        start_time = time.time()
        
        # Run all test suites
        tests = [
            self.test_api_health,
            self.test_api_status,
            self.test_database_connectivity,
            self.test_redis_connectivity,
            self.test_metrics_endpoints,
            self.test_environment_isolation,
            self.test_response_times
        ]
        
        passed_tests = 0
        total_tests = len(tests)
        
        for test in tests:
            try:
                if test():
                    passed_tests += 1
            except Exception as e:
                print(f"❌ Test failed with exception: {e}")
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Generate summary
        print("\n" + "=" * 50)
        print("📊 INTEGRATION TEST SUMMARY")
        print("=" * 50)
        print(f"Total Tests: {total_tests}")
        print(f"Passed: {passed_tests}")
        print(f"Failed: {total_tests - passed_tests}")
        print(f"Success Rate: {(passed_tests/total_tests)*100:.1f}%")
        print(f"Duration: {duration:.2f} seconds")
        
        # Detailed results
        print("\n📋 DETAILED RESULTS:")
        for result in self.results:
            print(f"{result['status']} {result['test']}")
            if result['details']:
                print(f"    {result['details']}")
        
        return {
            "total_tests": total_tests,
            "passed": passed_tests,
            "failed": total_tests - passed_tests,
            "success_rate": (passed_tests/total_tests)*100,
            "duration": duration,
            "results": self.results
        }

if __name__ == "__main__":
    # Check if required endpoints are available
    try:
        requests.get("http://localhost:8080/health", timeout=5)
    except:
        print("❌ Production API not available at http://localhost:8080")
        sys.exit(1)
    
    try:
        requests.get("http://localhost:8081/health", timeout=5)
    except:
        print("❌ Staging API not available at http://localhost:8081")
        sys.exit(1)
    
    # Run tests
    test_suite = IntegrationTestSuite()
    results = test_suite.run_all_tests()
    
    # Exit with appropriate code
    if results["failed"] > 0:
        print(f"\n❌ {results['failed']} tests failed!")
        sys.exit(1)
    else:
        print(f"\n✅ All {results['passed']} tests passed!")
        sys.exit(0)
