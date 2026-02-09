#!/bin/bash

API_URL="http://localhost:8080"

echo "=== Тестирование TODO API ==="
echo ""

echo "1. GET /todos - Получение списка задач (должен быть пустой):"
curl -s $API_URL/todos | jq .
echo -e "\n"

echo "2. POST /todos - Создание новой задачи:"
TODO1=$(curl -s -X POST $API_URL/todos \
  -H "Content-Type: application/json" \
  -d '{"task": "Изучить Rust", "is_finished": false}')
echo $TODO1 | jq .
TODO1_ID=$(echo $TODO1 | jq -r '.id')
echo -e "\n"

echo "3. POST /todos - Создание второй задачи:"
TODO2=$(curl -s -X POST $API_URL/todos \
  -H "Content-Type: application/json" \
  -d '{"task": "Написать REST API"}')
echo $TODO2 | jq .
TODO2_ID=$(echo $TODO2 | jq -r '.id')
echo -e "\n"

echo "4. GET /todos - Получение всех задач:"
curl -s $API_URL/todos | jq .
echo -e "\n"

echo "5. GET /todos/$TODO1_ID - Получение задачи по ID:"
curl -s $API_URL/todos/$TODO1_ID | jq .
echo -e "\n"

echo "6. PUT /todos/$TODO1_ID - Обновление задачи (отметить как выполненную):"
curl -s -X PUT $API_URL/todos/$TODO1_ID \
  -H "Content-Type: application/json" \
  -d '{"is_finished": true}' | jq .
echo -e "\n"

echo "7. PUT /todos/$TODO2_ID - Изменение текста задачи:"
curl -s -X PUT $API_URL/todos/$TODO2_ID \
  -H "Content-Type: application/json" \
  -d '{"task": "Написать REST API на Actix-Web", "is_finished": true}' | jq .
echo -e "\n"

echo "8. POST /todos - Создание третьей задачи:"
TODO3=$(curl -s -X POST $API_URL/todos \
  -H "Content-Type: application/json" \
  -d '{"task": "Развернуть в Docker"}')
TODO3_ID=$(echo $TODO3 | jq -r '.id')
echo $TODO3 | jq .
echo -e "\n"

echo "9. GET /todos - Все задачи после обновлений:"
curl -s $API_URL/todos | jq .
echo -e "\n"

echo "10. DELETE /todos/$TODO3_ID - Удаление задачи:"
curl -s -X DELETE $API_URL/todos/$TODO3_ID -w "\nHTTP Status: %{http_code}\n"
echo -e "\n"

echo "11. GET /todos - Финальный список задач:"
curl -s $API_URL/todos | jq .
echo -e "\n"

echo "12. GET /todos/999 - Попытка получить несуществующую задачу (404):"
curl -s $API_URL/todos/999 -w "\nHTTP Status: %{http_code}\n" | jq .
echo -e "\n"

echo "13. POST /todos - Валидация: пустая задача (должна вернуть ошибку):"
curl -s -X POST $API_URL/todos \
  -H "Content-Type: application/json" \
  -d '{"task": ""}' -w "\nHTTP Status: %{http_code}\n" | jq .
echo -e "\n"

echo "14. POST /todos - Валидация: слишком длинная задача (должна вернуть ошибку):"
LONG_TASK=$(printf 'a%.0s' {1..600})
curl -s -X POST $API_URL/todos \
  -H "Content-Type: application/json" \
  -d "{\"task\": \"$LONG_TASK\"}" -w "\nHTTP Status: %{http_code}\n" | jq .
echo -e "\n"

echo "=== Тест завершен ==="
