import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '30s', target: 10 },
    { duration: '1m', target: 50 },
    { duration: '2m', target: 50 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],
    errors: ['rate<0.05'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3002';

export default function () {
  group('Health Check', () => {
    const res = http.get(BASE_URL + '/health');
    check(res, { 'status 200': (r) => r.status === 200 });
    errorRate.add(res.status !== 200);
  });

  group('List Contacts', () => {
    const res = http.get(BASE_URL + '/api/contacts');
    check(res, { 'status 200': (r) => r.status === 200 });
    errorRate.add(res.status !== 200);
  });

  sleep(1);
}
