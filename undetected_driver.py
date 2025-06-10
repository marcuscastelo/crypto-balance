import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

class UndetectedDriver:
    def __init__(self):
        self.driver = uc.Chrome()
        self.initialized = False

    def get_profile_html(self, address):
        url = f"https://debank.com/profile/{address}"
        self.driver.get(url)
        WebDriverWait(self.driver, 30).until(
            EC.visibility_of_element_located((By.CSS_SELECTOR, "div.HeaderInfo_totalAssetInner__HyrdC"))
        )
        WebDriverWait(self.driver, 30).until(
            EC.text_to_be_present_in_element((By.XPATH, "/html/body/div[1]/div[1]/div[1]/div/div/div/div[2]/div/div[2]/div[2]/span"), "Data updated")
        )
        self.initialized = True
        return self.driver.page_source

    def click_chain(self, chain_index):
        if not self.initialized:
            raise Exception("Driver not initialized. Call get_profile_html first.")
        chains = self.driver.find_elements(By.CSS_SELECTOR, "div.AssetsOnChain_chainInfo__fKA2k")
        if chain_index < 0 or chain_index >= len(chains):
            raise IndexError("Invalid chain index")
        chains[chain_index].click()
        WebDriverWait(self.driver, 10).until(lambda d: True)  # Pequeno delay

    def get_expanded_html(self):
        if not self.initialized:
            raise Exception("Driver not initialized. Call get_profile_html first.")
        return self.driver.page_source

driver_instance = UndetectedDriver()

def get_profile_html(address):
    return driver_instance.get_profile_html(address)

def click_chain(chain_index):
    return driver_instance.click_chain(chain_index)

def get_expanded_html():
    return driver_instance.get_expanded_html()
