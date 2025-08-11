#[cfg(test)]
mod integration_tests {
    use std::{sync::Arc, time::Duration};

    use rust_p2p_node::{
        dht::{peer::PeerInfo, rpc::DhtRpc},
        helpers::{create_test_node, now},
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        task,
    };

    #[tokio::test]
    async fn test_two_nodes_communication() {
        let node1 = Arc::new(create_test_node(8091));
        let node2 = Arc::new(create_test_node(8092));

        let node2_clone = Arc::clone(&node2);
        let handle = task::spawn(async move {
            let listener = TcpListener::bind(node2_clone.addr).await.unwrap();
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let node = Arc::clone(&node2_clone);

                task::spawn(async move {
                    let mut len_buf = [0u8; 4];
                    socket.read_exact(&mut len_buf).await.unwrap();

                    let len = u32::from_be_bytes(len_buf) as usize;
                    let mut buf = vec![0u8; len];
                    socket.read_exact(&mut buf).await.unwrap();

                    let request: DhtRpc = bincode::deserialize(&buf).unwrap();
                    let response = node.handle_rpc(request).await;

                    let response_buf = bincode::serialize(&response).unwrap();
                    let len = (response_buf.len() as u32).to_be_bytes();
                    socket.write_all(&len).await.unwrap();
                    socket.write_all(&response_buf).await.unwrap();
                });
            }
        });

        // Даем время на запуск сервера
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Добавляем node2 в routing table node1
        let peer_info = PeerInfo {
            id: node2.id.clone(),
            addr: node2.addr,
            last_seen: now(),
        };
        node1.add_peer(peer_info);

        // Проверяем ping-pong
        let response = node1.send_rpc(node2.addr, DhtRpc::Ping).await.unwrap();
        assert!(matches!(response, DhtRpc::Pong));

        // Тестируем хранилище
        let key = b"shared_key".to_vec();
        let value = b"shared_value".to_vec();

        // Сохраняем на node1
        node1.store(key.clone(), value.clone()).await.unwrap();

        // Проверяем на node1
        let found = node1.find_value(key.clone()).await.unwrap();
        assert_eq!(found, value);

        // Проверяем на node2 (должно быть реплицировано)
        let found = node2.find_value(key.clone()).await.unwrap();
        assert_eq!(found, value);

        handle.abort();
    }
}
