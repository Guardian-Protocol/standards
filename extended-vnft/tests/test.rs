use extended_vnft_client::{
    traits::{ExtendedVnftFactory, Vnft},
    ExtendedVnftFactory as Factory, TokenMetadata, Vnft as VftClient,
};
use sails_rs::calls::*;
use sails_rs::gtest::{calls::*, System};

pub const ADMIN_ID: u64 = 10;
pub const USER_ID: [u64; 2] = [11, 12];

#[tokio::test]
async fn test_basic_function() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    system.mint_to(USER_ID[0], 1_000_000_000_000_000);
    system.mint_to(USER_ID[1], 1_000_000_000_000_000);
    let program_space = GTestRemoting::new(system, ADMIN_ID.into());

    let code_id = program_space
        .system()
        .submit_code_file("../target/wasm32-gear/release/extended_vnft.opt.wasm");

    let extended_vnft_factory = Factory::new(program_space.clone());
    let extended_vnft_id = extended_vnft_factory
        .new(
            "collection_name".to_string(),
            "collection_symbol".to_string(),
        )
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut client = VftClient::new(program_space);
    // mint
    let metadata = TokenMetadata {
        name: "token_name".to_string(),
        description: "token_description".to_string(),
        media: "token_media".to_string(),
        reference: "token_reference".to_string(),
    };
    client
        .mint(ADMIN_ID.into(), metadata)
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(ADMIN_ID.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1.into());
    // check token_id
    let token_id = client.token_id().recv(extended_vnft_id).await.unwrap();
    assert_eq!(token_id, 1.into());
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, ADMIN_ID.into());

    // transfer
    client
        .transfer(USER_ID[0].into(), 0.into())
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, USER_ID[0].into());

    // approve
    client
        .approve(USER_ID[1].into(), 0.into())
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vnft_id)
        .await
        .unwrap();

    // transfer from
    client
        .transfer_from(USER_ID[0].into(), ADMIN_ID.into(), 0.into())
        .with_args(GTestArgs::new(USER_ID[1].into()))
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, ADMIN_ID.into());

    // burn
    client
        .burn(ADMIN_ID.into(), 0.into())
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(ADMIN_ID.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, 0.into());
}

#[tokio::test]
async fn test_grant_role() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
    system.mint_to(USER_ID[0], 1_000_000_000_000_000);
    system.mint_to(USER_ID[1], 1_000_000_000_000_000);
    let program_space = GTestRemoting::new(system, ADMIN_ID.into());

    let mut client = VftClient::new(program_space.clone());

    let code_id = program_space
        .system()
        .submit_code_file("../target/wasm32-gear/release/extended_vnft.opt.wasm");

    let extended_vft_factory = Factory::new(program_space.clone());
    let extended_vft_id = extended_vft_factory
        .new("name".to_string(), "symbol".to_string())
        .send_recv(code_id, "123")
        .await
        .unwrap();

    // try minter role
    let metadata = TokenMetadata {
        name: "token_name".to_string(),
        description: "token_description".to_string(),
        media: "token_media".to_string(),
        reference: "token_reference".to_string(),
    };
    let res = client
        .mint(USER_ID[0].into(), metadata)
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant mint role
    client
        .grant_minter_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert!(minters.contains(&ADMIN_ID.into()));
    assert!(minters.contains(&USER_ID[0].into()));
    client
        .mint(
            USER_ID[0].into(),
            TokenMetadata {
                name: "token_name".to_string(),
                description: "token_description".to_string(),
                media: "token_media".to_string(),
                reference: "token_reference".to_string(),
            },
        )
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();

    let balance = client
        .balance_of(USER_ID[0].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1.into());

    // try burner role
    let res = client
        .burn(USER_ID[0].into(), 0.into())
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant burn role
    client
        .grant_burner_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert!(burners.contains(&ADMIN_ID.into()));
    assert!(burners.contains(&USER_ID[0].into()));
    client
        .burn(USER_ID[0].into(), 0.into())
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();

    let balance = client
        .balance_of(USER_ID[0].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());

    // grant admin role
    client
        .grant_admin_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert!(admins.contains(&ADMIN_ID.into()));
    assert!(admins.contains(&USER_ID[0].into()));
    // revoke roles
    client
        .revoke_admin_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert_eq!(admins, vec![ADMIN_ID.into()]);
    client
        .revoke_minter_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert_eq!(minters, vec![ADMIN_ID.into()]);
    client
        .revoke_burner_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert_eq!(burners, vec![ADMIN_ID.into()]);
}
