const cardContainer = document.getElementById('card-container');

        // 結果を表示する関数
        function displayResults(data) {
            cardContainer.innerHTML = ''; // Clear previous results
            const rawDataDisplay = document.getElementById('raw-data-display');
            rawDataDisplay.textContent = JSON.stringify(data, null, 2); // 生データを表示
            rawDataDisplay.style.display = 'block'; // 生データの表示を有効に
            console.log('Received data:', rawDataDisplay.textContent); // コンソールにデータを表示


            const items = Array.isArray(data) ? data : [data];

            items.forEach(item => {
                const itemDiv = document.createElement('div');
                itemDiv.classList.add('financial-card'); // 新しいクラスを追加

                const code = item.code || 'N/A (e.g., USDJPY=FX)';
                const heading = document.createElement('h4');
                heading.textContent = `Code: ${code}`;
                itemDiv.appendChild(heading);

                const pre = document.createElement('pre');
                pre.textContent = JSON.stringify(item, null, 2);
                itemDiv.appendChild(pre);

                cardContainer.appendChild(itemDiv);
            });
        }

        // デフォルト指標を取得して表示
        async function fetchDefaultQuotes() {
            cardContainer.textContent = 'デフォルト指標を取得中...'; // ここも変更
            try {
                const response = await fetch('/api/default');
                if (!response.ok) {
                    const errorText = await response.text();
                    throw new Error(`サーバーエラー: ${response.status} ${response.statusText} - ${errorText}`);
                }
                const data = await response.json();
                displayResults(data);
            } catch (error) {
                cardContainer.textContent = `エラー: ${error.message}`;
            }
        }

        // 個別コード検索のイベントリスナー
        document.getElementById('code-form').addEventListener('submit', async (event) => {
            event.preventDefault();
            const code = document.getElementById('code-input').value;
            cardContainer.textContent = `'${code}'を検索中...`; // ここも変更

            try {
                const response = await fetch('/api/quote', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ code: code })
                });

                if (!response.ok) {
                    const errorText = await response.text();
                    throw new Error(`サーバーエラー: ${response.status} ${response.statusText} - ${errorText}`);
                }

                const data = await response.json();
                displayResults(data);
            } catch (error) {
                cardContainer.textContent = `エラー: ${error.message}`;
            }
        });

        // ページ読み込み時にデフォルト指標を取得
        window.addEventListener('load', fetchDefaultQuotes);