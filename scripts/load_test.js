import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    vus: 50,
    duration: '30s',
    thresholds: {
        http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
        http_req_failed: ['rate<0.01'],   // http errors should be less than 1%
    },
};

export default function () {
    const res = http.get('http://localhost:8080/api/health');
    check(res, {
        'status is 200': (r) => r.status === 200,
        'content is healthy': (r) => r.body.includes('healthy'),
    });
    sleep(1);
}
