use asset::*;
use rate::*;
use std::collections::HashMap;
use std::ops;

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Clone, Copy)]
pub struct Quantity(pub i32);

#[derive(Debug, Clone)]
pub struct Account<TAsset: Asset>(pub HashMap<TAsset, Quantity>);

pub enum Tranx<TAsset: Asset> {
    Approved(Account<TAsset>, Account<TAsset>),
    Denied(HashMap<TAsset, Quantity>),
}

impl<TAsset: Asset> Account<TAsset> {
    pub fn quantity(&self, asset: &TAsset) -> Quantity {
        match self.0.get(asset) {
            Some(quantity) => quantity.clone(),
            None => Quantity(0),
        }
    }

    pub fn exchange(
        rate: &Rate<TAsset>,
        quantity: Quantity,
        buyer: &Account<TAsset>,
        seller: &Account<TAsset>,
    ) -> Tranx<TAsset> {
        let credit = &Account(rate.credit.clone()) * quantity;
        let debit = &Account(rate.debit.clone()) * quantity;
        let (buyer, seller) = (&(buyer - &debit) + &credit, &(seller - &credit) + &debit);
        let mut success = true;
        let mut deficit = hashmap![];
        {
            let Account(buyer) = &buyer;
            let Account(debit) = debit;
            for asset in debit.keys() {
                match buyer.get(asset) {
                    Some(Quantity(quantity)) if *quantity < 0 => {
                        success = false;
                        deficit.insert(asset.clone(), Quantity(*quantity));
                    }
                    _ => (),
                }
            }
        }
        if success {
            Tranx::Approved(buyer, seller)
        } else {
            Tranx::Denied(deficit)
        }
    }

    pub fn map(&self) -> &HashMap<TAsset, Quantity> {
        let Account(map) = self;
        map
    }

    fn prime(&mut self, rhs: &Account<TAsset>) {
        let Account(lhs) = self;
        let Account(rhs) = rhs;
        for rhs_key in rhs.keys() {
            if !lhs.contains_key(rhs_key) {
                lhs.insert(rhs_key.clone(), Quantity(0));
            }
        }
    }

    fn op<F>(lhs: &Account<TAsset>, rhs: &Account<TAsset>, op: F) -> Account<TAsset>
    where
        F: Fn(&Quantity, &Quantity) -> Quantity,
    {
        let mut acc = hashmap![];
        let mut lhs = lhs.clone();
        let mut rhs = rhs.clone();
        lhs.prime(&rhs);
        rhs.prime(&lhs);
        let Account(lhs) = lhs;
        let Account(rhs) = rhs;
        for key in lhs.keys() {
            let lhs_quantity = lhs.get(key).unwrap();
            let rhs_quantity = rhs.get(key).unwrap();
            let quantity = op(lhs_quantity, rhs_quantity);
            acc.insert(key.clone(), quantity.clone());
        }
        Account(acc)
    }
}

impl<TAsset: Asset> PartialEq for Account<TAsset> {
    fn eq(&self, rhs: &Account<TAsset>) -> bool {
        let mut lhs = self.clone();
        let mut rhs = rhs.clone();
        lhs.prime(&rhs);
        rhs.prime(&lhs);
        let Account(lhs) = lhs;
        let Account(rhs) = rhs;
        lhs == rhs
    }
}

impl<'a, 'b, TAsset: Asset> ops::Add<&'a Account<TAsset>> for &'b Account<TAsset> {
    type Output = Account<TAsset>;

    fn add(self, rhs: &Account<TAsset>) -> Account<TAsset> {
        Account::op(self, rhs, |Quantity(lq), Quantity(rq)| Quantity(lq + rq))
    }
}

impl<'a, 'b, TAsset: Asset> ops::Sub<&'a Account<TAsset>> for &'b Account<TAsset> {
    type Output = Account<TAsset>;

    fn sub(self, rhs: &Account<TAsset>) -> Account<TAsset> {
        Account::op(self, rhs, |Quantity(lq), Quantity(rq)| Quantity(lq - rq))
    }
}

impl<'a, TAsset: Asset> ops::Mul<Quantity> for &'a Account<TAsset> {
    type Output = Account<TAsset>;

    fn mul(self, rhs: Quantity) -> Account<TAsset> {
        let Account(lhs) = self;
        let keys = lhs.keys();
        let mut lhs = lhs.clone();
        let Quantity(rhs_quantity) = rhs;
        for key in keys {
            let q = lhs.entry(key.clone()).or_insert(Quantity(0));
            let Quantity(lhs_quantity) = *q;
            *q = Quantity(lhs_quantity * rhs_quantity);
        }
        Account(lhs)
    }
}
